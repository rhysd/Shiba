use crate::cli::Options;
use crate::renderer::{
    MenuItem as AppMenuItem, MenuItems, MessageFromRenderer, MessageToRenderer, Renderer, UserEvent,
};
use anyhow::Result;
use wry::application::accelerator::Accelerator;
use wry::application::event_loop::EventLoop;
use wry::application::keyboard::{KeyCode, ModifiersState};
use wry::application::menu::{AboutMetadata, MenuBar, MenuId, MenuItem, MenuItemAttributes};
use wry::application::window::{Window, WindowBuilder};
use wry::webview::{FileDropEvent, WebView, WebViewBuilder};

pub struct WebViewMenuItems {
    quit: MenuId,
    forward: MenuId,
    back: MenuId,
}

impl WebViewMenuItems {
    fn create(window: &Window) -> Self {
        let mut menu = MenuBar::new();

        let mut file_menu = MenuBar::new();
        file_menu.add_native_item(MenuItem::About("Shiba".to_string(), AboutMetadata::default()));
        let cmd_q = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyQ);
        let quit_item =
            file_menu.add_item(MenuItemAttributes::new("Quit").with_accelerators(&cmd_q));
        menu.add_submenu("File", true, file_menu);

        let mut history_menu = MenuBar::new();
        let cmd_left_bracket = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::BracketRight);
        let forward_item = history_menu
            .add_item(MenuItemAttributes::new("Forward").with_accelerators(&cmd_left_bracket));
        let cmd_right_bracket = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::BracketLeft);
        let back_item = history_menu
            .add_item(MenuItemAttributes::new("Back").with_accelerators(&cmd_right_bracket));
        menu.add_submenu("History", true, history_menu);

        window.set_menu(Some(menu));
        log::debug!("Added menubar to window");
        Self { quit: quit_item.id(), forward: forward_item.id(), back: back_item.id() }
    }
}

impl MenuItems for WebViewMenuItems {
    type ItemId = MenuId;

    fn item_from_id(&self, id: Self::ItemId) -> Result<AppMenuItem> {
        if id == self.quit {
            Ok(AppMenuItem::Quit)
        } else if id == self.forward {
            Ok(AppMenuItem::Forward)
        } else if id == self.back {
            Ok(AppMenuItem::Back)
        } else {
            Err(anyhow::anyhow!("Unknown item id: {:?}", id))
        }
    }
}

impl Renderer for WebView {
    type EventLoop = EventLoop<UserEvent>;
    type Menu = WebViewMenuItems;

    fn open(options: &Options, event_loop: &Self::EventLoop, html: &str) -> Result<Self> {
        let ipc_proxy = event_loop.create_proxy();
        let file_drop_proxy = event_loop.create_proxy();

        let window = WindowBuilder::new().with_title("Shiba").build(event_loop)?;
        log::debug!("Event loop and window were created successfully");

        let webview = WebViewBuilder::new(window)?
            .with_html(html)?
            .with_devtools(options.debug)
            .with_ipc_handler(move |_w, s| {
                let m: MessageFromRenderer = serde_json::from_str(&s).unwrap();
                log::debug!("Message from WebView: {:?}", m);
                if let Err(e) = ipc_proxy.send_event(UserEvent::IpcMessage(m)) {
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

        #[cfg(debug_assertions)]
        if options.debug {
            webview.open_devtools(); // This method is defined in debug build only
            log::debug!("Opened DevTools for debugging");
        }

        log::debug!("Created WebView successfully");
        Ok(webview)
    }

    fn set_menu(&self) -> Self::Menu {
        WebViewMenuItems::create(self.window())
    }

    fn send_message(&self, message: MessageToRenderer) -> Result<()> {
        let mut buf = b"window.myMarkdownPreview.receive(".to_vec();
        serde_json::to_writer(&mut buf, &message)?;
        buf.push(b')');
        self.evaluate_script(&String::from_utf8(buf).unwrap())?; // XXX: This UTF-8 validation is redundant
        Ok(())
    }
}
