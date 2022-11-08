use crate::assets::{assets, is_asset_path, AssetsLoaded};
use crate::cli::Options;
use crate::renderer::{
    MenuItem as AppMenuItem, MenuItems, MessageFromRenderer, MessageToRenderer, Renderer, UserEvent,
};
use anyhow::Result;
use std::cell::RefCell;
use std::path::PathBuf;
use wry::application::accelerator::Accelerator;
use wry::application::event_loop::EventLoop;
use wry::application::keyboard::{KeyCode, ModifiersState};
use wry::application::menu::{AboutMetadata, MenuBar, MenuId, MenuItem, MenuItemAttributes};
use wry::application::window::{Window, WindowBuilder};
use wry::http::header::CONTENT_TYPE;
use wry::http::Response;
use wry::webview::{FileDropEvent, WebView, WebViewBuilder};

pub struct WryMenuIds {
    open_file: MenuId,
    watch_dir: MenuId,
    quit: MenuId,
    forward: MenuId,
    back: MenuId,
    reload: MenuId,
    search: MenuId,
    search_next: MenuId,
    search_prev: MenuId,
}

impl WryMenuIds {
    fn setup(menu: &mut MenuBar) -> Self {
        let mut file_menu = MenuBar::new();
        let cmd_o = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyO);
        let open_file_item =
            file_menu.add_item(MenuItemAttributes::new("Open File...").with_accelerators(&cmd_o));
        let cmd_opt_o =
            Accelerator::new(Some(ModifiersState::SUPER | ModifiersState::ALT), KeyCode::KeyO);
        let watch_dir_item = file_menu
            .add_item(MenuItemAttributes::new("Watch Directory...").with_accelerators(&cmd_opt_o));
        file_menu.add_native_item(MenuItem::About("Shiba".to_string(), AboutMetadata::default()));
        let cmd_q = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyQ);
        let quit_item =
            file_menu.add_item(MenuItemAttributes::new("Quit").with_accelerators(&cmd_q));
        menu.add_submenu("File", true, file_menu);

        let mut edit_menu = MenuBar::new();
        let cmd_left_bracket = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::BracketRight);
        let forward_item = edit_menu
            .add_item(MenuItemAttributes::new("Forward").with_accelerators(&cmd_left_bracket));
        let cmd_right_bracket = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::BracketLeft);
        let back_item = edit_menu
            .add_item(MenuItemAttributes::new("Back").with_accelerators(&cmd_right_bracket));
        let cmd_f = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyF);
        let search_item =
            edit_menu.add_item(MenuItemAttributes::new("Search").with_accelerators(&cmd_f));
        let cmd_g = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyG);
        let search_next_item =
            edit_menu.add_item(MenuItemAttributes::new("Search Next").with_accelerators(&cmd_g));
        let cmd_shift_g =
            Accelerator::new(Some(ModifiersState::SUPER | ModifiersState::SHIFT), KeyCode::KeyG);
        let search_prev_item = edit_menu
            .add_item(MenuItemAttributes::new("Search Previous").with_accelerators(&cmd_shift_g));
        menu.add_submenu("Edit", true, edit_menu);

        let mut display_menu = MenuBar::new();
        let cmd_r = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyR);
        let reload_item =
            display_menu.add_item(MenuItemAttributes::new("Reload").with_accelerators(&cmd_r));
        menu.add_submenu("Display", true, display_menu);

        log::debug!("Added menubar to window");
        Self {
            open_file: open_file_item.id(),
            watch_dir: watch_dir_item.id(),
            quit: quit_item.id(),
            forward: forward_item.id(),
            back: back_item.id(),
            reload: reload_item.id(),
            search: search_item.id(),
            search_next: search_next_item.id(),
            search_prev: search_prev_item.id(),
        }
    }
}

impl MenuItems for WryMenuIds {
    type ItemId = MenuId;

    fn item_from_id(&self, id: Self::ItemId) -> Result<AppMenuItem> {
        if id == self.open_file {
            Ok(AppMenuItem::OpenFile)
        } else if id == self.watch_dir {
            Ok(AppMenuItem::WatchDir)
        } else if id == self.quit {
            Ok(AppMenuItem::Quit)
        } else if id == self.forward {
            Ok(AppMenuItem::Forward)
        } else if id == self.back {
            Ok(AppMenuItem::Back)
        } else if id == self.reload {
            Ok(AppMenuItem::Reload)
        } else if id == self.search {
            Ok(AppMenuItem::Search)
        } else if id == self.search_next {
            Ok(AppMenuItem::SearchNext)
        } else if id == self.search_prev {
            Ok(AppMenuItem::SearchPrevious)
        } else {
            Err(anyhow::anyhow!("Unknown menu item id: {:?}", id))
        }
    }
}

fn create_webview(window: Window, event_loop: &EventLoop<UserEvent>) -> Result<WebView> {
    let ipc_proxy = event_loop.create_proxy();
    let file_drop_proxy = event_loop.create_proxy();
    let navigation_proxy = event_loop.create_proxy();

    // These flags must be wrapped with `RefCell` since the handler callback is defined as `Fn`.
    // Dynamically borrowing the mutable value is mandatory. The `Fn` boundary is derived from
    // `webkit2gtk::WebView::connect_decide_policy` so it is difficult to change.
    // https://github.com/tauri-apps/webkit2gtk-rs/blob/cce947f86f2c0d50710c1ea9ea9f160c8b6cbf4a/src/auto/web_view.rs#L1249
    let assets_loaded = RefCell::new(AssetsLoaded::default());

    WebViewBuilder::new(window)?
        .with_url("shiba://localhost/")?
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
        .with_navigation_handler(move |mut url| {
            // Custom protocol URLs are different for each platform
            //   macOS, Linux → <scheme_name>://<path>
            //   Windows → https://<scheme_name>.<path>
            #[cfg(not(target_os = "windows"))]
            const CUSTOM_PROTOCOL_URL: &str = "shiba://localhost/";
            #[cfg(target_os = "windows")]
            const CUSTOM_PROTOCOL_URL: &str = "https://shiba.localhost/";

            let event = if url.starts_with(CUSTOM_PROTOCOL_URL) {
                log::debug!("Navigating to custom protocol URL {}", url);
                let path = &url[CUSTOM_PROTOCOL_URL.len() - 1..]; // `- 1` for first '/'

                if is_asset_path(path) && !assets_loaded.borrow_mut().is_loaded(path) {
                    return true;
                }

                url.drain(0..CUSTOM_PROTOCOL_URL.len()); // shiba://localhost/foo/bar -> foo/bar
                if url.is_empty() {
                    url.push('.');
                }

                UserEvent::OpenLocalPath(PathBuf::from(url))
            } else {
                log::debug!("Navigating to URL {}", url);
                UserEvent::OpenExternalLink(url)
            };

            if let Err(e) = navigation_proxy.send_event(event) {
                log::error!("Could not send navigation event: {}", e);
            }

            false // Don't allow navigating to any external links
        })
        .with_new_window_req_handler(|url| {
            log::debug!("Rejected to open new window for URL: {}", url);
            false
        })
        .with_custom_protocol("shiba".into(), |request| {
            let uri = request.uri();
            log::debug!("Handling custom protocol: {:?}", uri);
            let path = uri.path();
            let (body, mime) = assets(path);
            let status = if body.is_empty() { 404 } else { 200 };
            Response::builder()
                .status(status)
                .header(CONTENT_TYPE, mime)
                .body(body)
                .map_err(Into::into)
        })
        .build()
        .map_err(Into::into)
}

pub struct Wry {
    webview: WebView,
    menu_ids: WryMenuIds,
}

impl Renderer for Wry {
    type EventLoop = EventLoop<UserEvent>;
    type Menu = WryMenuIds;

    fn open(options: &Options, event_loop: &Self::EventLoop) -> Result<Self> {
        let mut menu = MenuBar::new();
        let menu_ids = WryMenuIds::setup(&mut menu);

        let window = WindowBuilder::new().with_title("Shiba").with_menu(menu).build(event_loop)?;
        log::debug!("Event loop and window were created successfully");

        let webview = create_webview(window, event_loop)?;
        log::debug!("WebView was created successfully with options: {:?}", options);

        #[cfg(debug_assertions)]
        if options.debug {
            webview.open_devtools(); // This method is defined in debug build only
            log::debug!("Opened DevTools for debugging");
        }

        Ok(Wry { webview, menu_ids })
    }

    fn menu(&self) -> &Self::Menu {
        &self.menu_ids
    }

    fn send_message(&self, message: MessageToRenderer) -> Result<()> {
        let mut buf = b"window.postShibaMessageFromMain(".to_vec();
        serde_json::to_writer(&mut buf, &message)?;
        buf.push(b')');
        self.webview.evaluate_script(&String::from_utf8(buf).unwrap())?; // XXX: This UTF-8 validation is redundant
        Ok(())
    }

    fn set_title(&self, title: &str) {
        log::debug!("Set window title: {}", title);
        self.webview.window().set_title(title);
    }
}
