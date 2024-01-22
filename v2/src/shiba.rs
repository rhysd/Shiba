use crate::cli::Options;
use crate::config::{Config, SearchMatcher};
use crate::dialog::Dialog;
use crate::markdown::{DisplayText, MarkdownContent, MarkdownParser};
use crate::opener::Opener;
use crate::renderer::{
    EventHandler, MenuItem, MessageFromRenderer, MessageToRenderer, Renderer, Rendering,
    RenderingFlow, UserEvent,
};
use crate::watcher::{PathFilter, Watcher};
use anyhow::{Context as _, Error, Result};
use std::collections::VecDeque;
use std::fs;
use std::marker::PhantomData;
use std::mem;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

enum Zoom {
    In,
    Out,
}

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

        if let Some(current) = self.current() {
            if current == &item {
                return; // Do not push the same path repeatedly
            }
        } else {
            log::debug!("Push first history item: {:?}", item);
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
        log::debug!("Push new history item at index {}: {:?}", self.index, item);
        self.items.push_back(item);
    }

    fn forward(&mut self) {
        if self.index + 1 < self.items.len() {
            self.index += 1;
        }
    }

    fn back(&mut self) {
        if let Some(i) = self.index.checked_sub(1) {
            self.index = i;
        }
    }

    fn next(&self) -> Option<&PathBuf> {
        self.items.get(self.index + 1)
    }

    fn prev(&self) -> Option<&PathBuf> {
        self.items.get(self.index.checked_sub(1)?)
    }

    fn current(&self) -> Option<&PathBuf> {
        self.items.get(self.index)
    }

    fn iter(&self) -> impl Iterator<Item = &'_ Path> {
        self.items.iter().map(PathBuf::as_path)
    }
}

struct PreviewContent {
    home_dir: Option<PathBuf>,
    content: MarkdownContent,
    text: DisplayText,
    title: String,
}

impl Default for PreviewContent {
    fn default() -> Self {
        let home_dir = dirs::home_dir();
        #[cfg(target_os = "windows")]
        let home_dir = home_dir.and_then(|p| p.canonicalize().ok()); // Ensure \\? at the head of the path
        Self {
            home_dir,
            content: MarkdownContent::default(),
            text: DisplayText::default(),
            title: String::new(),
        }
    }
}

impl PreviewContent {
    fn home_dir(&self) -> Option<&'_ Path> {
        self.home_dir.as_deref()
    }

    fn title(&self, path: &Path) -> String {
        if let Some(home_dir) = &self.home_dir {
            if let Ok(path) = path.strip_prefix(home_dir) {
                return format!("Shiba: ~{}{}", MAIN_SEPARATOR, path.display());
            }
        }
        format!("Shiba: {}", path.display())
    }

    pub fn show<R: Renderer>(&mut self, path: &Path, renderer: &R) -> Result<bool> {
        log::debug!("Opening markdown preview for {:?}", path);
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(err) => {
                // Do not return error 'no such file' because the file might be renamed and no longer
                // exists. This can happen when saving files on Vim. In this case, a file create event
                // will follow so the preview can be updated with the event.
                log::debug!("Could not open {:?} due to error: {}", path, err);
                return Ok(false);
            }
        };

        let title = self.title(path);
        let is_new = self.title != title;
        let new_content = MarkdownContent::new(source, path.parent());
        let prev_content = std::mem::replace(&mut self.content, new_content);
        let offset = if is_new { None } else { prev_content.modified_utf8_offset(&self.content) };
        log::debug!("Last modified offset: {:?}", offset);

        self.text = renderer.send_message_raw(MarkdownParser::new(&self.content, offset, ()))?;

        if is_new {
            renderer.set_title(&title);
            self.title = title;
            renderer.send_message(MessageToRenderer::PathChanged { path })?;
        }

        Ok(true)
    }

    pub fn rerender<R: Renderer>(&mut self, renderer: &R) -> Result<()> {
        renderer.send_message_raw(MarkdownParser::new(&self.content, None, ()))
    }

    pub fn search<R: Renderer>(
        &mut self,
        renderer: &R,
        query: &str,
        index: Option<usize>,
        matcher: SearchMatcher,
    ) -> Result<()> {
        log::debug!("Re-rendering content with query {:?} and current index {:?}", query, index);
        if query.is_empty() {
            return self.rerender(renderer);
        }

        let matches = match self.text.search(query, matcher) {
            Ok(m) => m,
            Err(err) => {
                log::debug!("Could not build {:?} matcher for query {:?}: {}", matcher, query, err);
                return self.rerender(renderer);
            }
        };
        log::debug!("Search hit {} matches", matches.len());

        let Some(tokenizer) = matches.tokenizer(index) else {
            return self.rerender(renderer);
        };
        renderer.send_message_raw(MarkdownParser::new(&self.content, None, tokenizer))
    }
}

pub struct Shiba<R: Rendering, O, W, D> {
    renderer: R::Renderer,
    opener: O,
    history: History,
    watcher: W,
    config: Config,
    preview: PreviewContent,
    init_file: Option<PathBuf>,
    last_error: Option<Error>,
    _dialog: PhantomData<D>,
}

impl<R, O, W, D> Shiba<R, O, W, D>
where
    R: Rendering,
    O: Opener,
    W: Watcher,
    D: Dialog,
{
    pub fn run(options: Options) -> Result<()> {
        fn on_err<D: Dialog>(err: Error) -> Error {
            let err = err.context("Could not launch application");
            D::alert(&err);
            err
        }
        let mut rendering = R::new().map_err(on_err::<D>)?;
        let dog = Self::new(options, &mut rendering).map_err(on_err::<D>)?;
        rendering.run(dog)
    }

    pub fn new(mut options: Options, rendering: &mut R) -> Result<Self> {
        log::debug!("Application options: {:?}", options);
        let watch_paths = mem::take(&mut options.watch_paths);
        let init_file = mem::take(&mut options.init_file);

        let config = Config::load(options)?;
        log::debug!("Application config: {:?}", config);

        let renderer = rendering.create_renderer(&config)?;

        let filter = PathFilter::new(config.watch());
        let mut watcher = W::new(rendering.create_sender(), filter)?;
        for path in watch_paths {
            log::debug!("Watching initial path: {:?}", path);
            watcher.watch(&path)?;
        }

        let mut history = History::new(History::DEFAULT_MAX_HISTORY_SIZE);
        for path in config.data_dir().load_recent_files(config.max_recent_files()) {
            history.push(path);
        }

        Ok(Self {
            renderer,
            opener: O::default(),
            history,
            watcher,
            config,
            preview: PreviewContent::default(),
            init_file,
            last_error: None,
            _dialog: PhantomData,
        })
    }

    fn preview_new(&mut self, path: PathBuf) -> Result<()> {
        self.watcher.watch(&path)?; // Watch path at first since the file may not exist yet
        if self.preview.show(&path, &self.renderer)? {
            self.history.push(path);
        }
        Ok(())
    }

    fn forward(&mut self) -> Result<()> {
        if let Some(path) = self.history.next() {
            log::debug!("Forward to next preview page: {:?}", path);
            self.preview.show(path, &self.renderer)?;
            self.history.forward();
        }
        Ok(())
    }

    fn back(&mut self) -> Result<()> {
        if let Some(path) = self.history.prev() {
            log::debug!("Back to previous preview page: {:?}", path);
            self.preview.show(path, &self.renderer)?;
            self.history.back();
        }
        Ok(())
    }

    fn reload(&mut self) -> Result<()> {
        if self.preview.content.is_empty() {
            // When content is empty, we don't need to reload the page. This happens when 'welcome' page displays just
            // after launching the app.
            log::debug!("Skipped to reload empty content");
            return Ok(());
        }
        if let Some(path) = self.history.current() {
            log::debug!("Reload current preview page: {:?}", path);
            self.preview.show(path, &self.renderer)?;
            self.renderer.send_message(MessageToRenderer::Reload)?;
        }
        Ok(())
    }

    fn open_file(&mut self) -> Result<()> {
        let extensions = self.config.watch().file_extensions();
        let dir = self.config.dialog().default_dir()?;
        let file = D::pick_file(&dir, extensions);
        #[cfg(target_os = "windows")]
        let file = file.and_then(|p| p.canonicalize().ok()); // Ensure \\? at the head of the path

        if let Some(file) = file {
            log::debug!("Previewing file chosen by dialog: {:?}", file);
            self.preview_new(file)?;
        }

        Ok(())
    }

    fn open_dir(&mut self) -> Result<()> {
        let dir = self.config.dialog().default_dir()?;
        let dir = D::pick_dir(&dir);
        #[cfg(target_os = "windows")]
        let dir = dir.and_then(|p| p.canonicalize().ok()); // Ensure \\? at the head of the path

        if let Some(dir) = dir {
            log::debug!("Watching directory chosen by dialog: {:?}", dir);
            self.watcher.watch(&dir)?;
        }

        Ok(())
    }

    fn zoom(&mut self, zoom: Zoom) -> Result<()> {
        let level = match zoom {
            Zoom::In => self.renderer.zoom_level().zoom_in(),
            Zoom::Out => self.renderer.zoom_level().zoom_out(),
        };

        let Some(level) = level else {
            return Ok(());
        };

        self.renderer.zoom(level);
        let percent = level.percent();
        log::debug!("Changed zoom factor: {}%", percent);
        self.renderer.send_message(MessageToRenderer::Zoomed { percent })?;

        Ok(())
    }

    fn toggle_always_on_top(&mut self) -> Result<()> {
        let pinned = !self.renderer.always_on_top();
        log::debug!("Toggle always-on-top (pinned={})", pinned);
        self.renderer.set_always_on_top(pinned);
        self.renderer.send_message(MessageToRenderer::AlwaysOnTop { pinned })
    }

    fn toggle_maximized(&mut self) {
        let maximized = !self.renderer.is_maximized();
        log::debug!("Toggle maximized window (maximized={})", maximized);
        self.renderer.set_maximized(maximized);
    }

    fn open_config(&mut self) -> Result<()> {
        let path = self.config.config_file()?;
        log::debug!("Opening config file via menu item: {:?}", path);
        self.opener.open(&path)
    }

    fn handle_ipc_message(&mut self, message: MessageFromRenderer) -> Result<RenderingFlow> {
        use MessageFromRenderer::*;
        match message {
            Init => {
                if self.config.debug() {
                    self.renderer.send_message(MessageToRenderer::Debug)?;
                }

                self.renderer.send_message(MessageToRenderer::Config {
                    keymaps: self.config.keymaps(),
                    search: self.config.search(),
                    theme: self.renderer.theme(),
                    recent: &self.history.iter().collect::<Vec<_>>(),
                    home: self.preview.home_dir(),
                    window: self.renderer.window_appearance(),
                })?;

                // Open window when the content is ready. Otherwise a white window flashes when dark theme.
                self.renderer.show();

                if let Some(path) = mem::take(&mut self.init_file) {
                    self.preview_new(path)?;
                } else {
                    self.renderer.send_message(MessageToRenderer::Welcome)?;
                }
            }
            Search { query, index, matcher } => {
                self.preview.search(&self.renderer, &query, index, matcher)?;
            }
            Forward => self.forward()?,
            Back => self.back()?,
            Reload => self.reload()?,
            FileDialog => self.open_file()?,
            DirDialog => self.open_dir()?,
            OpenFile { path } => {
                let path = PathBuf::from(path);
                if self.preview.show(&path, &self.renderer)? {
                    self.history.push(path);
                }
            }
            ZoomIn => self.zoom(Zoom::In)?,
            ZoomOut => self.zoom(Zoom::Out)?,
            DragWindow => self.renderer.drag_window()?,
            ToggleMaximized => self.toggle_maximized(),
            Quit => return Ok(RenderingFlow::Close),
            OpenMenu { position } => self.renderer.show_menu_at(position),
            ToggleMenuBar => self.renderer.toggle_menu()?,
            Error { message } => anyhow::bail!("Error reported from renderer: {}", message),
        }
        Ok(RenderingFlow::Continue)
    }
}

impl<R, O, W, D> EventHandler for Shiba<R, O, W, D>
where
    R: Rendering,
    O: Opener,
    W: Watcher,
    D: Dialog,
{
    fn handle_user_event(&mut self, event: UserEvent) -> Result<RenderingFlow> {
        log::debug!("Handling user event {:?}", event);
        match event {
            UserEvent::IpcMessage(msg) => return self.handle_ipc_message(msg),
            UserEvent::FileDrop(mut path) => {
                log::debug!("Previewing file dropped into window: {:?}", path);
                if !path.is_absolute() {
                    path = path.canonicalize()?;
                }
                self.preview_new(path)?;
            }
            UserEvent::WatchedFilesChanged(mut paths) => {
                log::debug!("Files changed: {:?}", paths);
                if let Some(current) = self.history.current() {
                    if paths.contains(current) {
                        self.preview.show(current, &self.renderer)?;
                        return Ok(RenderingFlow::Continue);
                    }
                }
                // Choose the last one to preview if the current file is not included in `paths`
                if let Some(mut path) = paths.pop() {
                    if !path.is_absolute() {
                        path = path.canonicalize()?;
                    }
                    if self.preview.show(&path, &self.renderer)? {
                        self.history.push(path);
                    }
                }
            }
            UserEvent::OpenLocalPath(mut path) => {
                if path.is_relative() {
                    if let Some(current_file) = self.history.current() {
                        if let Some(dir) = current_file.parent() {
                            path = dir.join(path).canonicalize()?;
                        }
                    }
                }
                let path = path;
                let is_markdown = self.config.watch().file_extensions().matches(&path);
                if is_markdown {
                    log::debug!("Opening local markdown link clicked in WebView: {:?}", path);
                    self.preview_new(path)?;
                } else {
                    log::debug!("Opening local link item clicked in WebView: {:?}", path);
                    self.opener.open(&path).with_context(|| format!("opening path {:?}", &path))?;
                }
            }
            UserEvent::OpenExternalLink(link) => {
                log::debug!("Opening external link item clicked in WebView: {:?}", link);
                self.opener.open(&link).with_context(|| format!("opening link {:?}", &link))?;
            }
            UserEvent::Error(err) => return Err(err),
        }
        Ok(RenderingFlow::Continue)
    }

    fn handle_menu_event(&mut self, item: MenuItem) -> Result<RenderingFlow> {
        use MenuItem::*;

        log::debug!("Menu item was clicked: {:?}", item);
        match item {
            Quit => return Ok(RenderingFlow::Close),
            Forward => self.forward()?,
            Back => self.back()?,
            Reload => self.reload()?,
            OpenFile => self.open_file()?,
            WatchDir => self.open_dir()?,
            Search => self.renderer.send_message(MessageToRenderer::Search)?,
            SearchNext => self.renderer.send_message(MessageToRenderer::SearchNext)?,
            SearchPrevious => self.renderer.send_message(MessageToRenderer::SearchPrevious)?,
            Outline => self.renderer.send_message(MessageToRenderer::Outline)?,
            Print => self.renderer.print()?,
            ZoomIn => self.zoom(Zoom::In)?,
            ZoomOut => self.zoom(Zoom::Out)?,
            #[cfg(not(target_os = "macos"))]
            ToggleMenuBar => self.renderer.toggle_menu()?,
            History => self.renderer.send_message(MessageToRenderer::History)?,
            ToggleAlwaysOnTop => self.toggle_always_on_top()?,
            Help => self.renderer.send_message(MessageToRenderer::Help)?,
            OpenRepo => self.opener.open("https://github.com/rhysd/Shiba")?,
            EditConfig => self.open_config()?,
        }
        Ok(RenderingFlow::Continue)
    }

    fn handle_close(&mut self) -> Result<()> {
        log::debug!("Handling application exit");
        let data_dir = self.config.data_dir();
        if self.config.window().restore {
            if let Some(state) = self.renderer.window_state() {
                log::debug!("Saving window state as persistent data: {:?}", state);
                data_dir.save(&state)?;
            }
        }
        data_dir.save_recent_files(self.history.iter(), self.config.max_recent_files())?;
        Ok(())
    }

    fn handle_error(&mut self, err: Error) -> RenderingFlow {
        D::alert(&err);
        self.last_error = Some(err);
        RenderingFlow::Continue
    }

    fn handle_exit(&mut self) -> Result<()> {
        if let Some(err) = self.last_error.take() {
            Err(err)
        } else {
            Ok(())
        }
    }
}
