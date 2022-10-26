use crate::cli::Options;
use crate::opener::Opener;
use crate::renderer::{
    MenuItem, MenuItems, MessageFromRenderer, MessageToRenderer, Renderer, UserEvent,
};
use crate::watcher::{PathFilter, Watcher};
use anyhow::Result;
use std::collections::VecDeque;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};

const MARKDOWN_EXTENSIONS: &[&str] = &["md", "mkd", "markdown"];

struct History {
    max_items: usize,
    index: usize,
    items: VecDeque<PathBuf>,
}

impl History {
    const DEFAULT_MAX_HISTORY_SIZE: usize = 20;

    fn new(max_items: usize) -> Self {
        Self { max_items, index: 0, items: VecDeque::new() }
    }

    fn push(&mut self, item: PathBuf) {
        if self.max_items == 0 {
            return;
        }

        if let Some(latest) = self.items.back() {
            if latest == &item {
                return; // Do not push the same path repeatedly
            }
        } else {
            self.items.push_back(item);
            return;
        }

        if self.items.len() == self.max_items {
            self.items.pop_front();
            self.index = self.index.saturating_sub(1);
        }

        if self.index + 1 < self.items.len() {
            self.items.truncate(self.index + 1);
        }

        self.index += 1;
        self.items.push_back(item);
    }

    fn forward(&mut self) -> Option<&Path> {
        if self.items.is_empty() || self.index + 1 == self.items.len() {
            return None;
        }
        self.index += 1;
        Some(&self.items[self.index])
    }

    fn back(&mut self) -> Option<&Path> {
        self.index = self.index.checked_sub(1)?;
        Some(&self.items[self.index])
    }

    fn current(&self) -> Option<&Path> {
        self.items.get(self.index).map(PathBuf::as_path)
    }
}

#[derive(Debug)]
pub enum AppControl {
    Continue,
    Exit,
}

pub struct App<R: Renderer, O: Opener, W: Watcher> {
    options: Options,
    renderer: R,
    menu: R::Menu,
    opener: O,
    history: History,
    watcher: W,
}

impl<R, O, W> App<R, O, W>
where
    R: Renderer,
    O: Opener,
    W: Watcher<ChannelCreator = R::EventLoop>,
{
    pub fn new(options: Options, event_loop: &R::EventLoop) -> Result<Self> {
        let renderer = R::open(&options, event_loop)?;
        let menu = renderer.set_menu();
        let opener = O::new();
        let history = History::new(History::DEFAULT_MAX_HISTORY_SIZE);
        let filter = PathFilter::new(MARKDOWN_EXTENSIONS);
        let mut watcher = W::new(event_loop, filter)?;
        if let Some(path) = &options.init_file {
            watcher.watch(path)?;
        }
        for path in &options.watch_dirs {
            watcher.watch(path)?;
        }
        Ok(Self { options, renderer, menu, opener, history, watcher })
    }

    fn preview(&self, path: &Path) -> Result<()> {
        log::debug!("Opening markdown preview for {:?}", path);
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                // Do not return error because 'no such file' because the file might be renamed and
                // no longer exists. This can happen when saving files on Vim. In this case, a file
                // create event will follow so the preview can be updated with the event.
                log::debug!("Could not open {:?} due to error: {}", path, err);
                return Ok(());
            }
        };
        let msg = MessageToRenderer::Content { content: &content };
        self.renderer.send_message(msg)?;
        Ok(())
    }

    pub fn handle_user_event(&mut self, event: UserEvent) -> Result<()> {
        match event {
            UserEvent::IpcMessage(msg) => match msg {
                MessageFromRenderer::Init => {
                    if let Some(path) = mem::take(&mut self.options.init_file) {
                        self.preview(&path)?;
                        self.history.push(path);
                    }
                }
                MessageFromRenderer::Open { link }
                    if link.starts_with("https://") || link.starts_with("http://") =>
                {
                    self.opener.open(&link)?;
                }
                MessageFromRenderer::Open { mut link } => {
                    if link.starts_with("file://") {
                        link.drain(.."file://".len());
                    }
                    #[cfg(target_os = "windows")]
                    {
                        link = link.replace('/', "\\");
                    }
                    let link = PathBuf::from(link);
                    let is_markdown = MARKDOWN_EXTENSIONS
                        .iter()
                        .any(|e| link.extension().map(|ext| ext == *e).unwrap_or(false));
                    if is_markdown {
                        let mut path = link;
                        if path.is_relative() {
                            if let Some(current_file) = self.history.current() {
                                if let Some(dir) = current_file.parent() {
                                    path = dir.join(path);
                                }
                            }
                        }
                        log::debug!("Opening markdown link clicked in WebView: {:?}", path);
                        self.preview(&path)?;
                        self.watcher.watch(&path)?;
                        self.history.push(path);
                    } else {
                        log::debug!("Opening link item clicked in WebView: {:?}", link);
                        self.opener.open(&link)?;
                    }
                }
            },
            UserEvent::FileDrop(mut path) => {
                log::debug!("Previewing file dropped into window: {:?}", path);
                if !path.is_absolute() {
                    path = path.canonicalize()?;
                }
                self.preview(&path)?;
                self.watcher.watch(&path)?;
                self.history.push(path);
            }
            UserEvent::WatchedFilesChanged(mut paths) => {
                log::debug!("Files changed: {:?}", paths);
                // Currently only the last file is referred for preview. Should we push other paths to the history?
                if let Some(path) = paths.pop() {
                    self.preview(&path)?;
                    self.history.push(path);
                }
            }
        }
        Ok(())
    }

    pub fn handle_menu_event(&mut self, id: <R::Menu as MenuItems>::ItemId) -> Result<AppControl> {
        let kind = self.menu.item_from_id(id)?;
        log::debug!("Menu item was clicked: {:?}", kind);
        match kind {
            MenuItem::Quit => Ok(AppControl::Exit),
            MenuItem::Forward => {
                if let Some(path) = self.history.forward().map(Path::to_path_buf) {
                    self.preview(&path)?;
                }
                Ok(AppControl::Continue)
            }
            MenuItem::Back => {
                if let Some(path) = self.history.back().map(Path::to_path_buf) {
                    self.preview(&path)?;
                }
                Ok(AppControl::Continue)
            }
        }
    }
}
