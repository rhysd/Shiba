use crate::cli::Options;
use crate::opener::Opener;
use crate::renderer::{MenuItems, MessageFromWebView, MessageToWebView, Renderer, UserEvent};
use anyhow::Result;
use std::collections::VecDeque;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};

const MARKDOWN_EXTENSIONS: &[&str] = &[".md", ".mkd", ".markdown"];

#[derive(Debug)]
pub enum AppControl {
    Continue,
    Exit,
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

        if self.items.len() == self.max_items {
            self.items.pop_front();
            self.index = self.index.saturating_sub(1);
        }

        if self.index < self.items.len() {
            self.items.truncate(self.index);
        }

        self.index += 1;
        self.items.push_back(item);
    }

    fn forward(&mut self) -> Option<&Path> {
        if self.index == self.items.len() {
            return None;
        }
        let item = &self.items[self.index];
        self.index += 1;
        Some(item)
    }

    fn back(&mut self) -> Option<&Path> {
        self.index = self.index.checked_sub(1)?;
        let item = &self.items[self.index];
        Some(item)
    }

    fn current(&self) -> Option<&Path> {
        self.items.get(self.index).map(PathBuf::as_path)
    }
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

    fn preview(&mut self, path: PathBuf) -> Result<()> {
        log::debug!("Opening markdown preview for {:?}", path);
        let content = fs::read_to_string(&path)?;
        let msg = MessageToWebView::Content { content: &content };
        self.renderer.send_message(msg)?;
        self.history.push(path);
        Ok(())
    }

    pub fn handle_user_event(&mut self, event: UserEvent) -> Result<()> {
        match event {
            UserEvent::FromWebView(msg) => match msg {
                MessageFromWebView::Init => {
                    if let Some(path) = mem::take(&mut self.options.init_file) {
                        self.preview(path)?;
                    }
                }
                MessageFromWebView::Open { link }
                    if link.starts_with("https://") || link.starts_with("http://") =>
                {
                    self.opener.open(&link)?;
                }
                MessageFromWebView::Open { mut link } => {
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
                        self.preview(path)?;
                    } else {
                        log::debug!("Opening link item clicked in WebView: {:?}", link);
                        self.opener.open(&link)?;
                    }
                }
            },
            UserEvent::FileDrop(path) => {
                log::debug!("Previewing file dropped into window: {:?}", path);
                self.preview(path)?;
            }
        }
        Ok(())
    }

    pub fn handle_menu(&self, id: <R::Menu as MenuItems>::ItemId) -> AppControl {
        if self.menu.is_quit(id) {
            log::debug!("'Quit' menu item was clicked");
            AppControl::Exit
        } else {
            AppControl::Continue
        }
    }
}
