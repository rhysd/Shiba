use crate::assets::Assets;
use crate::config::{Config, WindowTheme as ThemeConfig};
use crate::persistent::WindowState;
use crate::renderer::{
    MessageFromRenderer, MessageToRenderer, RawMessageWriter, Renderer, Theme as RendererTheme,
    UserEvent, WindowAppearance, ZoomLevel,
};
use crate::wry::menu::Menu;
use anyhow::{Context as _, Result};
use tao::dpi::{PhysicalPosition, PhysicalSize};
#[cfg(target_os = "macos")]
use tao::platform::macos::WindowBuilderExtMacOS as _;
#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix;
use tao::window::{Fullscreen, Theme, Window, WindowBuilder};
use wry::http::header::CONTENT_TYPE;
use wry::http::Response;
#[cfg(target_os = "linux")]
use wry::WebViewBuilderExtUnix;
use wry::{FileDropEvent, WebContext, WebView, WebViewBuilder};
#[cfg(target_os = "windows")]
use wry::{MemoryUsageLevel, WebViewBuilderExtWindows, WebViewExtWindows};

pub type EventLoop = tao::event_loop::EventLoop<UserEvent>;

#[cfg(not(target_os = "macos"))]
const ICON_RGBA: &[u8] = include_bytes!("../assets/icon_32x32.rgba");

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

fn create_webview(window: &Window, event_loop: &EventLoop, config: &Config) -> Result<WebView> {
    let ipc_proxy = event_loop.create_proxy();
    let file_drop_proxy = event_loop.create_proxy();
    let navigation_proxy = event_loop.create_proxy();
    let loader = Assets::new(config, window_theme(window));

    let user_dir = config.data_dir().path().map(|dir| dir.join("WebView"));
    log::debug!("WebView user data directory: {:?}", user_dir);
    let mut context = WebContext::new(user_dir);

    #[cfg(not(target_os = "linux"))]
    let mut builder = WebViewBuilder::new(window);
    #[cfg(target_os = "linux")]
    let mut builder = WebViewBuilder::new_gtk(window.default_vbox().unwrap());

    builder = builder
        .with_url("shiba://localhost/index.html")?
        .with_ipc_handler(move |msg| {
            let msg: MessageFromRenderer = serde_json::from_str(&msg).unwrap();
            log::debug!("Message from WebView: {msg:?}");
            if let Err(err) = ipc_proxy.send_event(UserEvent::IpcMessage(msg)) {
                log::error!("Could not send user event for message from WebView: {err}");
            }
        })
        .with_file_drop_handler(move |event| {
            if let FileDropEvent::Dropped { paths, .. } = event {
                log::debug!("Files were dropped (the first one will be opened): {paths:?}",);
                if let Some(path) = paths.into_iter().next() {
                    if let Err(err) = file_drop_proxy.send_event(UserEvent::FileDrop(path)) {
                        log::error!("Could not send user event for file drop: {err}");
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
            const CUSTOM_PROTOCOL_URL: &str = "http://shiba.localhost/";

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
            Response::builder().status(status).header(CONTENT_TYPE, mime).body(body).unwrap_or_else(
                |err| {
                    log::error!("Could not build response for request {:?}: {:?}", uri, err);
                    Response::builder()
                        .status(404)
                        .header(CONTENT_TYPE, "application/octet-stream")
                        .body(vec![].into())
                        .unwrap()
                },
            )
        })
        .with_web_context(&mut context)
        .with_focused(true)
        .with_devtools(cfg!(any(debug_assertions, feature = "devtools")));

    #[cfg(target_os = "windows")]
    {
        use wry::Theme;
        builder = builder.with_browser_accelerator_keys(false);
        match config.window().theme {
            ThemeConfig::System => {}
            ThemeConfig::Dark => builder = builder.with_theme(Theme::Dark),
            ThemeConfig::Light => builder = builder.with_theme(Theme::Light),
        }
    }

    #[cfg(target_os = "macos")]
    {
        builder = builder.with_transparent(true);
    }

    let webview = builder.build()?;

    #[cfg(target_os = "macos")]
    {
        use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
        apply_vibrancy(window, NSVisualEffectMaterial::Sidebar, None, None)?;
    }

    Ok(webview)
}

pub struct WebViewRenderer {
    webview: WebView,
    window: Window,
    zoom_level: ZoomLevel,
    always_on_top: bool,
    menu: Menu,
}

impl WebViewRenderer {
    pub fn new(config: &Config, event_loop: &EventLoop, mut menu: Menu) -> Result<Self> {
        let mut builder = WindowBuilder::new().with_title("Shiba").with_visible(false);

        let window_state = if config.window().restore { config.data_dir().load() } else { None };
        let (zoom_level, always_on_top) = if let Some(state) = window_state {
            let WindowState {
                height,
                width,
                x,
                y,
                fullscreen,
                zoom_level,
                always_on_top,
                maximized,
            } = state;
            log::debug!("Restoring window state {state:?}");
            let size = PhysicalSize { width, height };
            builder = builder.with_inner_size(size);
            let position = PhysicalPosition { x, y };
            builder = builder.with_position(position);
            if fullscreen {
                builder = builder.with_fullscreen(Some(Fullscreen::Borderless(None)));
            } else if maximized {
                builder = builder.with_maximized(true);
            }
            (zoom_level, always_on_top)
        } else {
            if let Some(size) = config.window().default_size {
                let size = PhysicalSize { width: size.width, height: size.height };
                builder = builder.with_inner_size(size);
            }
            (ZoomLevel::default(), config.window().always_on_top)
        };

        if always_on_top {
            builder = builder.with_always_on_top(true);
        }

        match config.window().theme {
            ThemeConfig::System => {}
            ThemeConfig::Dark => builder = builder.with_theme(Some(Theme::Dark)),
            ThemeConfig::Light => builder = builder.with_theme(Some(Theme::Light)),
        }

        #[cfg(not(target_os = "macos"))]
        {
            use tao::window::Icon;
            let icon = Icon::from_rgba(ICON_RGBA.into(), 32, 32).unwrap();
            builder = builder.with_window_icon(Some(icon));
        }

        #[cfg(target_os = "macos")]
        {
            builder = builder
                .with_transparent(true)
                .with_fullsize_content_view(true)
                .with_titlebar_transparent(true)
                .with_title_hidden(true);
        }

        let window = builder.build(event_loop)?;
        if cfg!(target_os = "macos") || config.window().menu_bar {
            menu.toggle(&window)?;
        }

        let webview = create_webview(&window, event_loop, config)?;
        log::debug!("WebView was created successfully");

        let zoom_factor = zoom_level.factor();
        if zoom_factor != 1.0 {
            webview.zoom(zoom_factor);
            log::debug!("Zoom factor was set to {}", zoom_factor);
        }

        #[cfg(any(debug_assertions, feature = "devtools"))]
        if config.debug() {
            webview.open_devtools(); // This method is defined in debug build only
            log::debug!("Opened DevTools for debugging");
        }

        Ok(WebViewRenderer { webview, window, zoom_level, always_on_top, menu })
    }
}

impl Renderer for WebViewRenderer {
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

    #[cfg(not(target_os = "macos"))]
    fn set_title(&self, title: &str) {
        log::debug!("Set window title: {}", title);
        self.window.set_title(title);
    }
    #[cfg(target_os = "macos")]
    fn set_title(&self, _title: &str) {} // On macOS, the title bar is hidden

    fn window_state(&self) -> Option<WindowState> {
        let PhysicalSize { width, height } = self.window.inner_size();
        let PhysicalPosition { x, y } = match self.window.outer_position() {
            Ok(position) => position,
            Err(err) => {
                log::debug!("Could not get window position for window state: {}", err);
                return None;
            }
        };
        let fullscreen = self.window.fullscreen().is_some();
        let zoom_level = self.zoom_level;
        let always_on_top = self.always_on_top;
        let maximized = self.window.is_maximized();
        Some(WindowState { width, height, x, y, fullscreen, zoom_level, always_on_top, maximized })
    }

    fn theme(&self) -> RendererTheme {
        window_theme(&self.window)
    }

    fn show(&self) {
        self.window.set_visible(true);
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

    fn set_always_on_top(&mut self, enabled: bool) {
        if self.always_on_top != enabled {
            self.window.set_always_on_top(enabled);
            self.always_on_top = enabled;
        }
    }

    fn always_on_top(&self) -> bool {
        self.always_on_top
    }

    fn drag_window(&self) -> Result<()> {
        self.window.drag_window().context("Could not start dragging the window")
    }

    fn is_maximized(&self) -> bool {
        self.window.is_maximized() // Note: Window is unmaximized when a user changes the window size manually
    }

    fn set_maximized(&mut self, maximized: bool) {
        self.window.set_maximized(maximized);
    }

    fn window_appearance(&self) -> WindowAppearance {
        WindowAppearance {
            title: cfg!(not(target_os = "macos")),
            vibrancy: cfg!(target_os = "macos"),
            scroll_bar: cfg!(target_os = "macos"),
            border_top: cfg!(target_os = "windows"),
        }
    }

    fn show_menu_at(&self, position: Option<(f64, f64)>) {
        self.menu.show_at(position, &self.window);
    }

    fn toggle_menu(&mut self) -> Result<()> {
        self.menu.toggle(&self.window)
    }

    #[cfg(target_os = "windows")]
    fn set_active(&mut self, is_active: bool) {
        let level = if is_active { MemoryUsageLevel::Normal } else { MemoryUsageLevel::Low };
        log::debug!("Meory usage level is set to {level:?}");
        self.webview.set_memory_usage_level(level);
    }
    #[cfg(not(target_os = "windows"))]
    fn set_active(&mut self, _is_active: bool) {}
}
