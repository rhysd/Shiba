use crate::cli::Options;
use crate::opener::Opener;
use crate::renderer::{MenuItems, MessageFromWebView, MessageToWebView, Renderer, UserEvent};
use anyhow::Result;
use std::fs;
use std::path::Path;

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
}

impl<R: Renderer, O: Opener> App<R, O> {
    pub fn new(options: Options, event_loop: &R::EventLoop) -> Result<Self> {
        let renderer = R::open(&options, event_loop)?;
        let menu = renderer.set_menu();
        let opener = O::new();
        Ok(Self { options, renderer, menu, opener })
    }

    fn preview(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        log::debug!("Opening markdown preview for {:?}", path);
        let content = fs::read_to_string(path)?;
        let msg = MessageToWebView::Content { content: &content };
        self.renderer.send_message(msg)
    }

    pub fn handle_user_event(&self, event: UserEvent) -> Result<()> {
        match event {
            UserEvent::FromWebView(msg) => match msg {
                MessageFromWebView::Init => {
                    if let Some(path) = &self.options.init_file {
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
                        #[cfg(target_os = "windows")]
                        {
                            link = link.replace('/', "\\");
                        }
                    }
                    log::debug!("Opening link item clicked in WebView: {:?}", link);
                    // TODO: Open markdown document in this app
                    self.opener.open(&link)?;
                }
            },
            UserEvent::FileDrop(path) => {
                log::debug!("Previewing file dropped into window: {:?}", path);
                self.preview(&path)?;
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
