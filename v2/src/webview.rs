use crate::assets::Assets;
use crate::cli::Options;
use crate::config::{Config, WindowTheme as ThemeConfig};
use crate::persistent::WindowState;
use crate::renderer::{
    MenuItem as AppMenuItem, MenuItems, MessageFromRenderer, MessageToRenderer, RawMessageWriter,
    Renderer, Theme as RendererTheme, UserEvent,
};
use anyhow::Result;
use std::path::PathBuf;
use wry::application::accelerator::Accelerator;
use wry::application::dpi::{PhysicalPosition, PhysicalSize, Position, Size};
use wry::application::event_loop::EventLoop;
use wry::application::keyboard::{KeyCode, ModifiersState};
use wry::application::menu::{AboutMetadata, MenuBar, MenuId, MenuItem, MenuItemAttributes};
use wry::application::window::{Fullscreen, Theme, Window, WindowBuilder};
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
    outline: MenuId,
}

impl WryMenuIds {
    fn set_menu(root_menu: &mut MenuBar) -> Self {
        // Windows / macOS / Android / iOS: The metadata is ignored on these platforms.
        #[cfg(target_os = "linux")]
        let metadata = AboutMetadata {
            version: Some("2.0.0-alpha".into()),
            authors: Some(vec!["rhysd <lin90162@yahoo.co.jp>".into()]),
            copyright: Some("Copyright (c) 2015 rhysd".into()),
            license: Some("MIT".into()),
            website: Some("https://github.com/rhysd/Shiba".into()),
            ..Default::default()
        };
        #[cfg(not(target_os = "linux"))]
        let metadata = AboutMetadata::default();

        let mut file_menu = MenuBar::new();
        let cmd_o = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyO);
        let open_file_item =
            file_menu.add_item(MenuItemAttributes::new("Open File...").with_accelerators(&cmd_o));
        let cmd_opt_o =
            Accelerator::new(Some(ModifiersState::SUPER | ModifiersState::ALT), KeyCode::KeyO);
        let watch_dir_item = file_menu
            .add_item(MenuItemAttributes::new("Watch Directory...").with_accelerators(&cmd_opt_o));
        file_menu.add_native_item(MenuItem::Separator);
        file_menu.add_native_item(MenuItem::About("Shiba".to_string(), metadata));
        file_menu.add_native_item(MenuItem::Separator);
        file_menu.add_native_item(MenuItem::Services);
        file_menu.add_native_item(MenuItem::Separator);
        file_menu.add_native_item(MenuItem::Hide);
        file_menu.add_native_item(MenuItem::HideOthers);
        file_menu.add_native_item(MenuItem::ShowAll);
        file_menu.add_native_item(MenuItem::Separator);
        let cmd_q = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyQ);
        let quit_item =
            file_menu.add_item(MenuItemAttributes::new("Quit").with_accelerators(&cmd_q));
        root_menu.add_submenu("File", true, file_menu);

        let mut edit_menu = MenuBar::new();
        edit_menu.add_native_item(MenuItem::Undo);
        edit_menu.add_native_item(MenuItem::Redo);
        edit_menu.add_native_item(MenuItem::Separator);
        edit_menu.add_native_item(MenuItem::Cut);
        edit_menu.add_native_item(MenuItem::Copy);
        edit_menu.add_native_item(MenuItem::Paste);
        edit_menu.add_native_item(MenuItem::SelectAll);
        edit_menu.add_native_item(MenuItem::Separator);
        let cmd_f = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyF);
        let search_item =
            edit_menu.add_item(MenuItemAttributes::new("Search…").with_accelerators(&cmd_f));
        let cmd_g = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyG);
        let search_next_item =
            edit_menu.add_item(MenuItemAttributes::new("Search Next").with_accelerators(&cmd_g));
        let cmd_shift_g =
            Accelerator::new(Some(ModifiersState::SUPER | ModifiersState::SHIFT), KeyCode::KeyG);
        let search_prev_item = edit_menu
            .add_item(MenuItemAttributes::new("Search Previous").with_accelerators(&cmd_shift_g));
        let cmd_s = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyS);
        let outline_item = edit_menu
            .add_item(MenuItemAttributes::new("Section Outline…").with_accelerators(&cmd_s));
        root_menu.add_submenu("Edit", true, edit_menu);

        let mut display_menu = MenuBar::new();
        let cmd_r = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyR);
        let reload_item =
            display_menu.add_item(MenuItemAttributes::new("Reload").with_accelerators(&cmd_r));
        display_menu.add_native_item(MenuItem::Separator);
        display_menu.add_native_item(MenuItem::EnterFullScreen);
        root_menu.add_submenu("Display", true, display_menu);

        let mut history_menu = MenuBar::new();
        let cmd_left_bracket = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::BracketRight);
        let forward_item = history_menu
            .add_item(MenuItemAttributes::new("Forward").with_accelerators(&cmd_left_bracket));
        let cmd_right_bracket = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::BracketLeft);
        let back_item = history_menu
            .add_item(MenuItemAttributes::new("Back").with_accelerators(&cmd_right_bracket));
        root_menu.add_submenu("History", true, history_menu);

        let mut window_menu = MenuBar::new();
        window_menu.add_native_item(MenuItem::Minimize);
        window_menu.add_native_item(MenuItem::Zoom);
        root_menu.add_submenu("Window", true, window_menu);

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
            outline: outline_item.id(),
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
        } else if id == self.outline {
            Ok(AppMenuItem::Outline)
        } else {
            Err(anyhow::anyhow!("Unknown menu item id: {:?}", id))
        }
    }
}

fn window_theme(window: &Window) -> RendererTheme {
    match window.theme() {
        Theme::Light => RendererTheme::Light,
        Theme::Dark => RendererTheme::Dark,
        t => {
            log::error!("Unknown window theme: {:?}", t);
            RendererTheme::Dark
        }
    }
}

fn create_webview(window: Window, event_loop: &EventLoop<UserEvent>) -> Result<WebView> {
    let ipc_proxy = event_loop.create_proxy();
    let file_drop_proxy = event_loop.create_proxy();
    let navigation_proxy = event_loop.create_proxy();
    let assets = Assets::default();
    let theme = window_theme(&window);

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

                if assets.is_asset(path) {
                    return true;
                }

                url.drain(0..CUSTOM_PROTOCOL_URL.len()); // shiba://localhost/foo/bar -> foo/bar
                if url.is_empty() {
                    url.push('.');
                }

                if url.starts_with('#') {
                    log::debug!("Allow navigating to hash link {}", url); // For footnotes
                    return true;
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
        .with_custom_protocol("shiba".into(), move |request| {
            let uri = request.uri();
            log::debug!("Handling custom protocol: {:?}", uri);
            let path = uri.path();
            let (body, mime) = Assets::load(path, theme);
            let status = if body.is_empty() { 404 } else { 200 };
            // Response body of custom protocol handler requires `Vec<u8>`
            Response::builder()
                .status(status)
                .header(CONTENT_TYPE, mime)
                .body(body.to_vec())
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

    fn new(
        options: &Options,
        config: &Config,
        event_loop: &Self::EventLoop,
        window_state: Option<WindowState>,
    ) -> Result<Self> {
        let mut menu = MenuBar::new();
        let menu_ids = WryMenuIds::set_menu(&mut menu);

        let mut builder =
            WindowBuilder::new().with_title("Shiba").with_menu(menu).with_visible(false);
        if let Some(state) = window_state {
            log::debug!("Restoring window state {state:?}");
            let size = PhysicalSize { width: state.width, height: state.height };
            builder = builder.with_inner_size(Size::Physical(size));
            let position = PhysicalPosition { x: state.x, y: state.y };
            builder = builder.with_position(Position::Physical(position));
            if state.fullscreen {
                builder = builder.with_fullscreen(Some(Fullscreen::Borderless(None)));
            }
        }
        match config.window().theme {
            ThemeConfig::System => {}
            ThemeConfig::Dark => builder = builder.with_theme(Some(Theme::Dark)),
            ThemeConfig::Light => builder = builder.with_theme(Some(Theme::Light)),
        }
        let window = builder.build(event_loop)?;
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

    fn send_message_raw<W: RawMessageWriter>(&self, writer: W) -> Result<W::Output> {
        let mut buf = "window.postShibaMessageFromMain(".to_string();
        let result = writer.write_to(&mut buf)?;
        buf.push(')');
        self.webview.evaluate_script(&buf)?;
        Ok(result)
    }

    fn set_title(&self, title: &str) {
        log::debug!("Set window title: {}", title);
        self.webview.window().set_title(title);
    }

    fn window_state(&self) -> Option<WindowState> {
        let w = self.webview.window();
        let PhysicalPosition { x, y } = match w.inner_position() {
            Ok(position) => position,
            Err(err) => {
                log::debug!("Could not get window position for window state: {}", err);
                return None;
            }
        };
        let PhysicalSize { width, height } = w.inner_size();
        let fullscreen = w.fullscreen().is_some();
        Some(WindowState { width, height, x, y, fullscreen })
    }

    fn theme(&self) -> RendererTheme {
        window_theme(self.webview.window())
    }

    fn show(&self) {
        self.webview.window().set_visible(true);
    }
}
