use crate::cli::Options;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use wry::application::accelerator::Accelerator;
use wry::application::event_loop::EventLoop;
use wry::application::keyboard::{KeyCode, ModifiersState};
use wry::application::menu::{AboutMetadata, MenuBar, MenuId, MenuItem, MenuItemAttributes};
use wry::application::window::WindowBuilder;
use wry::webview::{FileDropEvent, WebView, WebViewBuilder};

const HTML: &str = include_str!("bundle.html");

#[derive(Serialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageToWebView<'a> {
    Content { content: &'a str },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageFromWebView {
    Init,
    Open { link: String },
}

#[derive(Debug)]
pub enum UserEvent {
    FromWebView(MessageFromWebView),
    FileDrop(PathBuf),
}

pub trait MenuItems {
    type ItemId;
    fn is_quit(&self, id: Self::ItemId) -> bool;
}

pub struct WebViewMenuItems {
    quit: MenuId,
}

impl MenuItems for WebViewMenuItems {
    type ItemId = MenuId;

    fn is_quit(&self, id: Self::ItemId) -> bool {
        id == self.quit
    }
}

pub trait Renderer: Sized {
    type EventLoop;
    type Menu: MenuItems;

    fn open(options: &Options, event_loop: &Self::EventLoop) -> Result<Self>;
    fn set_menu(&self) -> Self::Menu;
    fn send_message(&self, message: MessageToWebView) -> Result<()>;
}

impl Renderer for WebView {
    type EventLoop = EventLoop<UserEvent>;
    type Menu = WebViewMenuItems;

    fn open(options: &Options, event_loop: &Self::EventLoop) -> Result<Self> {
        let ipc_proxy = event_loop.create_proxy();
        let file_drop_proxy = event_loop.create_proxy();

        let window = WindowBuilder::new().with_title("Shiba").build(event_loop)?;
        log::debug!("Event loop and window were created successfully");

        let webview = WebViewBuilder::new(window)?
            .with_html(HTML)?
            .with_devtools(options.debug)
            .with_ipc_handler(move |_w, s| {
                let m: MessageFromWebView = serde_json::from_str(&s).unwrap();
                log::debug!("Message from WebView: {:?}", m);
                if let Err(e) = ipc_proxy.send_event(UserEvent::FromWebView(m)) {
                    log::error!("Could not send user event for message from WebView: {}", e);
                }
            })
            .with_file_drop_handler(move |_w, e| {
                if let FileDropEvent::Dropped(paths) = e {
                    log::debug!("Files were dropped (the first one will be opened): {:?}", paths);
                    if let Some(path) = paths.into_iter().next() {
                        if let Err(e) = file_drop_proxy.send_event(UserEvent::FileDrop(path)) {
                            log::error!("Could not send user event for file drop: {}", e);
                        }
                    }
                }
                true
            })
            .build()?;

        log::debug!("Created WebView successfully");
        Ok(webview)
    }

    fn set_menu(&self) -> Self::Menu {
        let mut menu = MenuBar::new();
        let mut sub_menu = MenuBar::new();
        sub_menu.add_native_item(MenuItem::About(
            "Markdown Preview".to_string(),
            AboutMetadata::default(),
        ));
        let quit_item = sub_menu.add_item(
            MenuItemAttributes::new("Quit")
                .with_accelerators(&Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyQ)),
        );
        menu.add_submenu("File", true, sub_menu);
        self.window().set_menu(Some(menu));
        log::debug!("Added menubar to window (quit={:?})", quit_item);
        WebViewMenuItems { quit: quit_item.id() }
    }

    fn send_message(&self, message: MessageToWebView) -> Result<()> {
        let mut buf = b"window.myMarkdownPreview.receive(".to_vec();
        serde_json::to_writer(&mut buf, &message)?;
        buf.push(b')');
        self.evaluate_script(&String::from_utf8(buf).unwrap())?; // XXX: This UTF-8 validation is redundant
        Ok(())
    }
}
