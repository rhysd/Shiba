use crate::cli::Options;
use crate::dialog::Dialog;
use crate::opener::Opener;
use crate::renderer::{
    MenuItem, MenuItems, MessageFromRenderer, MessageToRenderer, Renderer, UserEvent,
};
use crate::watcher::{PathFilter, WatchChannelCreator, Watcher};
use anyhow::Result;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::marker::PhantomData;
use std::mem;
use std::path::{Path, PathBuf};

const HTML: &str = include_str!("bundle.html");
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

    fn forward(&mut self) -> bool {
        if self.items.is_empty() || self.index + 1 == self.items.len() {
            return false;
        }
        self.index += 1;
        true
    }

    fn back(&mut self) -> bool {
        if let Some(i) = self.index.checked_sub(1) {
            self.index = i;
            true
        } else {
            false
        }
    }

    fn current(&self) -> Option<&PathBuf> {
        self.items.get(self.index)
    }
}

#[derive(Debug)]
pub enum AppControl {
    Continue,
    Exit,
}

pub struct App<R: Renderer, O: Opener, W: Watcher, D: Dialog> {
    options: Options,
    renderer: R,
    menu: R::Menu,
    opener: O,
    history: History,
    watcher: W,
    _dialog: PhantomData<D>,
}

impl<R, O, W, D> App<R, O, W, D>
where
    R: Renderer,
    O: Opener,
    W: Watcher,
    D: Dialog,
    R::EventLoop: WatchChannelCreator,
{
    pub fn new(options: Options, event_loop: &R::EventLoop) -> Result<Self> {
        let renderer = R::open(&options, event_loop, HTML)?;
        let menu = renderer.set_menu();
        let opener = O::default();
        let history = History::new(History::DEFAULT_MAX_HISTORY_SIZE);
        let filter = PathFilter::new(MARKDOWN_EXTENSIONS);
        let mut watcher = W::new(event_loop, filter)?;
        for path in &options.watch_dirs {
            log::debug!("Watching initial directory: {:?}", path);
            watcher.watch(path)?;
        }
        Ok(Self { options, renderer, menu, opener, history, watcher, _dialog: PhantomData })
    }

    fn preview(&self, path: &Path) -> Result<bool> {
        log::debug!("Opening markdown preview for {:?}", path);
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                // Do not return error because 'no such file' because the file might be renamed and
                // no longer exists. This can happen when saving files on Vim. In this case, a file
                // create event will follow so the preview can be updated with the event.
                log::debug!("Could not open {:?} due to error: {}", path, err);
                return Ok(false);
            }
        };
        let msg = MessageToRenderer::Content { content: &content };
        self.renderer.send_message(msg)?;
        Ok(true)
    }

    fn preview_new(&mut self, path: PathBuf) -> Result<()> {
        self.watcher.watch(&path)?; // Watch path at first since the file may not exist yet
        if self.preview(&path)? {
            self.history.push(path);
        }
        Ok(())
    }

    fn forward(&mut self) -> Result<()> {
        if self.history.forward() {
            if let Some(path) = self.history.current() {
                log::debug!("Forward to next preview page: {:?}", path);
                self.preview(path)?;
            }
        }
        Ok(())
    }

    fn back(&mut self) -> Result<()> {
        if self.history.back() {
            if let Some(path) = self.history.current() {
                log::debug!("Back to previous preview page: {:?}", path);
                self.preview(path)?;
            }
        }
        Ok(())
    }

    fn reload(&mut self) -> Result<()> {
        if let Some(path) = self.history.current() {
            log::debug!("Reload current preview page: {:?}", path);
            self.preview(path)?;
        }
        Ok(())
    }

    fn open_file(&mut self) -> Result<()> {
        // Should we use directory of the current file?
        let cwd = env::current_dir()?;
        if let Some(path) = D::pick_file(&cwd, MARKDOWN_EXTENSIONS) {
            log::debug!("Previewing file chosen by dialog: {:?}", path);
            self.preview_new(path)?;
        }
        Ok(())
    }

    fn open_dir(&mut self) -> Result<()> {
        // Should we use directory of the current file?
        let cwd = env::current_dir()?;
        if let Some(path) = D::pick_dir(&cwd) {
            log::debug!("Watching directory chosen by dialog: {:?}", path);
            self.watcher.watch(&path)?;
        }
        Ok(())
    }

    fn handle_ipc_message(&mut self, message: MessageFromRenderer) -> Result<()> {
        match message {
            MessageFromRenderer::Init => {
                if self.options.debug {
                    self.renderer.send_message(MessageToRenderer::Debug)?;
                }
                self.renderer.send_message(MessageToRenderer::default_key_mappings())?;
                if let Some(path) = mem::take(&mut self.options.init_file) {
                    self.preview_new(path)?;
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
                    self.preview_new(path)?;
                } else {
                    log::debug!("Opening link item clicked in WebView: {:?}", link);
                    self.opener.open(&link)?;
                }
            }
            MessageFromRenderer::Forward => self.forward()?,
            MessageFromRenderer::Back => self.back()?,
            MessageFromRenderer::Reload => self.reload()?,
            MessageFromRenderer::FileDialog => self.open_file()?,
            MessageFromRenderer::DirDialog => self.open_dir()?,
        }
        Ok(())
    }

    pub fn handle_user_event(&mut self, event: UserEvent) -> Result<()> {
        match event {
            UserEvent::IpcMessage(msg) => self.handle_ipc_message(msg),
            UserEvent::FileDrop(mut path) => {
                log::debug!("Previewing file dropped into window: {:?}", path);
                if !path.is_absolute() {
                    path = path.canonicalize()?;
                }
                self.preview_new(path)?;
                Ok(())
            }
            UserEvent::WatchedFilesChanged(mut paths) => {
                log::debug!("Files changed: {:?}", paths);
                if let Some(current) = self.history.current() {
                    if paths.contains(current) {
                        self.preview(current)?;
                        return Ok(());
                    }
                }
                // Choose the last one to preview if the current file is not included in `paths`
                if let Some(path) = paths.pop() {
                    if self.preview(&path)? {
                        self.history.push(path);
                    }
                }
                Ok(())
            }
            UserEvent::Error(err) => Err(err),
        }
    }

    pub fn handle_menu_event(&mut self, id: <R::Menu as MenuItems>::ItemId) -> Result<AppControl> {
        let kind = self.menu.item_from_id(id)?;
        log::debug!("Menu item was clicked: {:?}", kind);
        match kind {
            MenuItem::Quit => return Ok(AppControl::Exit),
            MenuItem::Forward => self.forward()?,
            MenuItem::Back => self.back()?,
            MenuItem::Reload => self.reload()?,
            MenuItem::OpenFile => self.open_file()?,
            MenuItem::WatchDir => self.open_dir()?,
        }
        Ok(AppControl::Continue)
    }
}
