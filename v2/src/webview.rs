use crate::assets::Assets;
use crate::cli::Options;
use crate::config::{Config, WindowTheme as ThemeConfig};
use crate::persistent::WindowState;
use crate::renderer::{
    MenuItem as AppMenuItem, MenuItems, MessageFromRenderer, MessageToRenderer, RawMessageWriter,
    Renderer, Theme as RendererTheme, UserEvent, ZoomLevel,
};
use anyhow::Result;
use std::collections::HashMap;
use wry::application::accelerator::Accelerator;
use wry::application::dpi::{PhysicalPosition, PhysicalSize, Position, Size};
use wry::application::event_loop::EventLoop;
use wry::application::keyboard::{KeyCode, ModifiersState};
#[cfg(not(target_os = "windows"))]
use wry::application::menu::AboutMetadata;
use wry::application::menu::{MenuBar, MenuId, MenuItem, MenuItemAttributes};
use wry::application::window::{Fullscreen, Theme, Window, WindowBuilder};
use wry::http::header::CONTENT_TYPE;
use wry::http::Response;
#[cfg(target_os = "windows")]
use wry::webview::WebViewBuilderExtWindows;
use wry::webview::{FileDropEvent, WebView, WebViewBuilder};

pub struct WryMenuIds(HashMap<MenuId, AppMenuItem>);

impl WryMenuIds {
    fn set_menu(root_menu: &mut MenuBar) -> Self {
        #[cfg(target_os = "macos")]
        const MOD: ModifiersState = ModifiersState::SUPER;
        #[cfg(not(target_os = "macos"))]
        const MOD: ModifiersState = ModifiersState::CONTROL;

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
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        let metadata = AboutMetadata::default();

        // Note: Some native menu items are not supported by Windows. Those items are actually not inserted into menu bar.

        let mut file_menu = MenuBar::new();
        let cmd_o = Accelerator::new(Some(MOD), KeyCode::KeyO);
        let open_file =
            file_menu.add_item(MenuItemAttributes::new("Open File…").with_accelerators(&cmd_o));
        let cmd_shift_o = Accelerator::new(Some(MOD | ModifiersState::SHIFT), KeyCode::KeyO);
        let watch_dir = file_menu
            .add_item(MenuItemAttributes::new("Watch Directory…").with_accelerators(&cmd_shift_o));
        file_menu.add_native_item(MenuItem::Separator);
        let print = file_menu.add_item(MenuItemAttributes::new("Print…"));
        file_menu.add_native_item(MenuItem::Separator);
        #[cfg(not(target_os = "windows"))]
        {
            file_menu.add_native_item(MenuItem::About("Shiba".to_string(), metadata));
            file_menu.add_native_item(MenuItem::Separator);
            file_menu.add_native_item(MenuItem::Services);
            file_menu.add_native_item(MenuItem::Separator);
        }
        file_menu.add_native_item(MenuItem::Hide);
        file_menu.add_native_item(MenuItem::HideOthers);
        file_menu.add_native_item(MenuItem::ShowAll);
        file_menu.add_native_item(MenuItem::Separator);
        let cmd_q = Accelerator::new(Some(MOD), KeyCode::KeyQ);
        let quit = file_menu.add_item(MenuItemAttributes::new("Quit").with_accelerators(&cmd_q));
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
        let cmd_f = Accelerator::new(Some(MOD), KeyCode::KeyF);
        let search =
            edit_menu.add_item(MenuItemAttributes::new("Search…").with_accelerators(&cmd_f));
        let cmd_g = Accelerator::new(Some(MOD), KeyCode::KeyG);
        let search_next =
            edit_menu.add_item(MenuItemAttributes::new("Search Next").with_accelerators(&cmd_g));
        let cmd_shift_g = Accelerator::new(Some(MOD | ModifiersState::SHIFT), KeyCode::KeyG);
        let search_prev = edit_menu
            .add_item(MenuItemAttributes::new("Search Previous").with_accelerators(&cmd_shift_g));
        let cmd_s = Accelerator::new(Some(MOD), KeyCode::KeyS);
        let outline = edit_menu
            .add_item(MenuItemAttributes::new("Section Outline…").with_accelerators(&cmd_s));
        root_menu.add_submenu("Edit", true, edit_menu);

        let mut display_menu = MenuBar::new();
        let cmd_r = Accelerator::new(Some(MOD), KeyCode::KeyR);
        let reload =
            display_menu.add_item(MenuItemAttributes::new("Reload").with_accelerators(&cmd_r));
        display_menu.add_native_item(MenuItem::Separator);
        #[cfg(not(target_os = "windows"))]
        {
            display_menu.add_native_item(MenuItem::EnterFullScreen);
            display_menu.add_native_item(MenuItem::Separator);
        }
        let cmd_plus = Accelerator::new(Some(MOD), KeyCode::Plus);
        let zoom_in =
            display_menu.add_item(MenuItemAttributes::new("Zoom In").with_accelerators(&cmd_plus));
        let cmd_minus = Accelerator::new(Some(MOD), KeyCode::Minus);
        let zoom_out = display_menu
            .add_item(MenuItemAttributes::new("Zoom Out").with_accelerators(&cmd_minus));
        root_menu.add_submenu("Display", true, display_menu);

        let mut history_menu = MenuBar::new();
        let cmd_left_bracket = Accelerator::new(Some(MOD), KeyCode::BracketRight);
        let forward = history_menu
            .add_item(MenuItemAttributes::new("Forward").with_accelerators(&cmd_left_bracket));
        let cmd_right_bracket = Accelerator::new(Some(MOD), KeyCode::BracketLeft);
        let back = history_menu
            .add_item(MenuItemAttributes::new("Back").with_accelerators(&cmd_right_bracket));
        history_menu.add_native_item(MenuItem::Separator);
        let cmd_y = Accelerator::new(Some(MOD), KeyCode::KeyY);
        let history =
            history_menu.add_item(MenuItemAttributes::new("History…").with_accelerators(&cmd_y));
        root_menu.add_submenu("History", true, history_menu);

        let mut window_menu = MenuBar::new();
        window_menu.add_native_item(MenuItem::Minimize);
        window_menu.add_native_item(MenuItem::Zoom);
        root_menu.add_submenu("Window", true, window_menu);

        let mut help_menu = MenuBar::new();
        let guide = help_menu.add_item(MenuItemAttributes::new("Show Guide…"));
        let open_repo = help_menu.add_item(MenuItemAttributes::new("Open Repository Page"));
        root_menu.add_submenu("Help", true, help_menu);

        log::debug!("Added menubar to window");

        #[rustfmt::skip]
        let ids = HashMap::from_iter({
            use AppMenuItem::*;

            [
                (open_file.id(),   OpenFile),
                (watch_dir.id(),   WatchDir),
                (quit.id(),        Quit),
                (forward.id(),     Forward),
                (back.id(),        Back),
                (reload.id(),      Reload),
                (search.id(),      Search),
                (search_next.id(), SearchNext),
                (search_prev.id(), SearchPrevious),
                (outline.id(),     Outline),
                (print.id(),       Print),
                (zoom_in.id(),     ZoomIn),
                (zoom_out.id(),    ZoomOut),
                (history.id(),     History),
                (guide.id(),       Help),
                (open_repo.id(),   OpenRepo),
            ]
        });

        Self(ids)
    }
}

impl MenuItems for WryMenuIds {
    type ItemId = MenuId;

    fn item_from_id(&self, id: Self::ItemId) -> Result<AppMenuItem> {
        if let Some(item) = self.0.get(&id).copied() {
            Ok(item)
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
            RendererTheme::Light
        }
    }
}

fn create_webview(
    window: Window,
    event_loop: &EventLoop<UserEvent>,
    config: &Config,
) -> Result<WebView> {
    let ipc_proxy = event_loop.create_proxy();
    let file_drop_proxy = event_loop.create_proxy();
    let navigation_proxy = event_loop.create_proxy();
    let loader = Assets::new(config, window_theme(&window));

    #[allow(unused_mut)]
    let mut builder = WebViewBuilder::new(window)?
        .with_url("shiba://localhost/index.html")?
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
                if &url[CUSTOM_PROTOCOL_URL.len()..] == "index.html" {
                    return true;
                }

                url.drain(0..CUSTOM_PROTOCOL_URL.len() - 1); // shiba://localhost/foo/bar -> /foo/bar

                if url.starts_with("/index.html#") {
                    log::debug!("Allow navigating to hash link {}", url);
                    return true;
                }

                if url.is_empty() {
                    url.push('.');
                }

                #[cfg(not(target_os = "windows"))]
                let path = url.into();
                #[cfg(target_os = "windows")]
                let path = url.replace('/', "\\").into();

                log::debug!("Opening local path {:?}", path);
                UserEvent::OpenLocalPath(path)
            } else {
                log::debug!("Navigating to URL {:?}", url);
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
            let (body, mime) = loader.load(path);
            let status = if body.is_empty() { 404 } else { 200 };
            // Response body of custom protocol handler requires `Vec<u8>`
            Response::builder()
                .status(status)
                .header(CONTENT_TYPE, mime)
                .body(body)
                .map_err(Into::into)
        })
        .with_devtools(cfg!(any(debug_assertions, feature = "devtools")));

    #[cfg(target_os = "windows")]
    {
        builder = builder.with_browser_accelerator_keys(false);
    }

    builder.build().map_err(Into::into)
}

pub struct Wry {
    webview: WebView,
    menu_ids: WryMenuIds,
    zoom_level: ZoomLevel,
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

        let zoom_level = if let Some(state) = window_state {
            log::debug!("Restoring window state {state:?}");
            let size = PhysicalSize { width: state.width, height: state.height };
            builder = builder.with_inner_size(Size::Physical(size));
            let position = PhysicalPosition { x: state.x, y: state.y };
            builder = builder.with_position(Position::Physical(position));
            if state.fullscreen {
                builder = builder.with_fullscreen(Some(Fullscreen::Borderless(None)));
            }
            state.zoom_level
        } else {
            ZoomLevel::default()
        };

        match config.window().theme {
            ThemeConfig::System => {}
            ThemeConfig::Dark => builder = builder.with_theme(Some(Theme::Dark)),
            ThemeConfig::Light => builder = builder.with_theme(Some(Theme::Light)),
        }

        if config.window().always_on_top {
            builder = builder.with_always_on_top(true);
        }

        let window = builder.build(event_loop)?;
        log::debug!("Event loop and window were created successfully");

        let webview = create_webview(window, event_loop, config)?;
        log::debug!("WebView was created successfully with options: {:?}", options);

        if zoom_level.factor() != 1.0 {
            webview.zoom(zoom_level.factor());
        }

        #[cfg(any(debug_assertions, feature = "devtools"))]
        if options.debug {
            webview.open_devtools(); // This method is defined in debug build only
            log::debug!("Opened DevTools for debugging");
        }

        Ok(Wry { webview, menu_ids, zoom_level })
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
        let mut buf = b"window.postShibaMessageFromMain(".to_vec();
        let result = writer.write_to(&mut buf)?;
        buf.push(b')');
        self.webview.evaluate_script(&String::from_utf8(buf).unwrap())?;
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
        let zoom_level = self.zoom_level;
        Some(WindowState { width, height, x, y, fullscreen, zoom_level })
    }

    fn theme(&self) -> RendererTheme {
        window_theme(self.webview.window())
    }

    fn show(&self) {
        self.webview.window().set_visible(true);
    }

    fn set_background_color(&self, rgba: (u8, u8, u8, u8)) -> Result<()> {
        self.webview.set_background_color(rgba)?;
        Ok(())
    }

    fn print(&self) -> Result<()> {
        Ok(self.webview.print()?)
    }

    fn zoom(&mut self, level: ZoomLevel) {
        self.webview.zoom(level.factor());
        self.zoom_level = level;
    }

    fn zoom_level(&self) -> ZoomLevel {
        self.zoom_level
    }
}
