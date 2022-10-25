use crate::cli::Options;
use crate::opener::Opener;
use crate::renderer::{
    MenuItem, MenuItems, MessageFromRenderer, MessageToRenderer, Renderer, UserEvent,
};
use anyhow::Result;
use std::collections::VecDeque;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};

const MARKDOWN_EXTENSIONS: &[&str] = &[".md", ".mkd", ".markdown"];

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

        if self.items.is_empty() {
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

pub struct App<R: Renderer, O: Opener> {
    options: Options,
    renderer: R,
    menu: R::Menu,
    opener: O,
    history: History,
}

impl<R: Renderer, O: Opener> App<R, O> {
    pub fn new(options: Options, event_loop: &R::EventLoop) -> Result<Self> {
        let renderer = R::open(&options, event_loop)?;
        let menu = renderer.set_menu();
        let opener = O::new();
        let history = History::new(History::DEFAULT_MAX_HISTORY_SIZE);
        Ok(Self { options, renderer, menu, opener, history })
    }

    fn preview(&self, path: &Path) -> Result<()> {
        log::debug!("Opening markdown preview for {:?}", path);
        let content = fs::read_to_string(&path)?;
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
                    if MARKDOWN_EXTENSIONS.iter().any(|e| link.ends_with(e)) {
                        let mut path = PathBuf::from(link);
                        if path.is_relative() {
                            if let Some(current_file) = self.history.current() {
                                if let Some(dir) = current_file.parent() {
                                    path = dir.join(path);
                                }
                            }
                        }
                        log::debug!("Opening markdown link clicked in WebView: {:?}", path);
                        self.preview(&path)?;
                        self.history.push(path);
                    } else {
                        log::debug!("Opening link item clicked in WebView: {:?}", link);
                        self.opener.open(&link)?;
                    }
                }
            },
            UserEvent::FileDrop(path) => {
                log::debug!("Previewing file dropped into window: {:?}", path);
                self.preview(&path)?;
                self.history.push(path);
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
