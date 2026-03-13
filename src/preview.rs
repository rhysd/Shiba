use crate::config::SearchMatcher;
use crate::markdown::{DisplayText, MarkdownContent, MarkdownParser};
use crate::renderer::{MessageToWindow, Window};
use anyhow::Result;
use std::fs;
use std::mem;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

pub struct Preview {
    home_dir: Option<PathBuf>,
    content: MarkdownContent,
    text: DisplayText,
    path: PathBuf,
}

impl Default for Preview {
    fn default() -> Self {
        let home_dir = dirs::home_dir();
        #[cfg(target_os = "windows")]
        let home_dir = home_dir.and_then(|p| p.canonicalize().ok()); // Ensure \\? at the head of the path
        Self {
            home_dir,
            content: MarkdownContent::default(),
            text: DisplayText::default(),
            path: PathBuf::new(),
        }
    }
}

impl Preview {
    pub fn home_dir(&self) -> Option<&'_ Path> {
        self.home_dir.as_deref()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn title(&self) -> String {
        if let Some(home_dir) = &self.home_dir
            && let Ok(path) = self.path.strip_prefix(home_dir)
        {
            return format!("Shiba: ~{}{}", MAIN_SEPARATOR, path.display());
        }
        format!("Shiba: {}", self.path.display())
    }

    pub fn show<W: Window>(&mut self, path: &Path, window: &W) -> Result<bool> {
        log::debug!("Opening markdown preview for {:?}", path);
        let new_content = match fs::read_to_string(path) {
            Ok(source) => MarkdownContent::new(source, path.parent()),
            Err(err) => {
                // Do not return error 'no such file' because the file might be renamed and no longer
                // exists. This can happen when saving files on Vim. In this case, a file create event
                // will follow so the preview can be updated with the event.
                log::debug!("Could not open {:?} due to error: {}", path, err);
                return Ok(false);
            }
        };

        let is_new = self.path != path;
        let prev_content = mem::replace(&mut self.content, new_content);
        let offset = if is_new { None } else { prev_content.modified_utf8_offset(&self.content) };
        log::debug!("Last modified offset: {:?}", offset);

        self.text = window.send_message_raw(MarkdownParser::new(&self.content, offset, ()))?;

        if is_new {
            window.send_message(MessageToWindow::Path { path })?;
            self.path = path.to_path_buf();
            let title = self.title();
            log::debug!("Preview title changed to {title:?}");
            window.set_title(&title);
        }

        Ok(true)
    }

    pub fn rerender<W: Window>(&self, window: &W) -> Result<()> {
        window.send_message_raw(MarkdownParser::new(&self.content, None, ()))
    }

    pub fn search<W: Window>(
        &self,
        window: &W,
        query: &str,
        index: Option<usize>,
        matcher: SearchMatcher,
    ) -> Result<()> {
        log::debug!("Re-rendering content with query {:?} and current index {:?}", query, index);
        if query.is_empty() {
            return self.rerender(window);
        }

        let matches = match self.text.search(query, matcher) {
            Ok(matches) => matches,
            Err(err) => {
                log::debug!("Could not build {:?} matcher for query {:?}: {}", matcher, query, err);
                return self.rerender(window);
            }
        };
        log::debug!("Search hit {} matches", matches.len());

        let Some(tokenizer) = matches.tokenizer(index) else {
            return self.rerender(window);
        };
        window.send_message_raw(MarkdownParser::new(&self.content, None, tokenizer))
    }
}
