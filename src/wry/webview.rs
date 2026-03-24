use crate::assets::Assets;
use crate::config::{Config, WindowLength, WindowTheme as ThemeConfig};
use crate::renderer::{
    Event, InitFile, MessageFromWindow, MessageToWindow, RawMessageWriter, Request,
    Window as RendererWindow, WindowAppearance, WindowHandles, WindowState, ZoomLevel,
};
use crate::wry::menu::WindowMenu;
use crate::wry::monitor::MonitorExtWorkArea as _;
use crate::wry::types::{EventLoop, Proxy};
use anyhow::{Context as _, Result};
use std::num::NonZeroU32;
use tao::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
#[cfg(target_os = "macos")]
use tao::platform::macos::WindowBuilderExtMacOS as _;
#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix;
#[cfg(target_os = "windows")]
use tao::platform::windows::WindowBuilderExtWindows as _;
#[cfg(target_os = "windows")]
use tao::platform::windows::WindowExtWindows as _;
use tao::window::{Fullscreen, Theme, Window, WindowBuilder, WindowId};
#[cfg(target_os = "linux")]
use wry::WebViewBuilderExtUnix;
use wry::http::Response;
use wry::http::header::CONTENT_TYPE;
use wry::{DragDropEvent, NewWindowResponse, WebContext, WebView, WebViewBuilder};
#[cfg(target_os = "windows")]
use wry::{MemoryUsageLevel, WebViewBuilderExtWindows, WebViewExtWindows};

#[cfg(any(target_os = "windows", target_os = "linux"))]
const ICON_RGBA: &[u8] = include_bytes!("../assets/icon_32x32.rgba");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MaximizeWindow {
    Vertical { width: NonZeroU32 },
    Horizontal { height: NonZeroU32 },
}

impl MaximizeWindow {
    fn maximize(self, window: &Window) {
        let Some(monitor) = window.current_monitor().or_else(|| window.primary_monitor()) else {
            log::error!(
                "Could not maximize window {self:?} because current/primary monitor is unavailable for {:?}",
                window.id(),
            );
            return;
        };

        let factor = monitor.scale_factor();
        let (monitor_size, monitor_pos) = monitor.work_area();
        let outer_size = window.outer_size();
        let inner_size = window.inner_size();
        let (size, pos) = match self {
            Self::Vertical { width } => {
                let width = (width.get() as f64 * factor) as u32;
                let height =
                    monitor_size.height - (outer_size.height.saturating_sub(inner_size.height));
                let x = monitor_pos.x + (monitor_size.width as i32 / 2) - width as i32 / 2;
                let y = monitor_pos.y;
                (PhysicalSize { width, height }, PhysicalPosition { x, y })
            }
            Self::Horizontal { height } => {
                let height = (height.get() as f64 * factor) as u32;
                let width =
                    monitor_size.width - (outer_size.width.saturating_sub(inner_size.width));
                let y = monitor_pos.y + (monitor_size.height as i32 / 2) - height as i32 / 2;
                let x = monitor_pos.x;
                (PhysicalSize { width, height }, PhysicalPosition { x, y })
            }
        };

        log::debug!("Resize window to size {size:?} at position {pos:?}");
        window.set_inner_size(size);
        window.set_outer_position(pos);
    }
}

fn create_window(event_loop: &EventLoop, config: &Config) -> Result<(Window, ZoomLevel, bool)> {
    let mut builder = WindowBuilder::new()
        .with_title("Shiba")
        .with_visible(false)
        .with_min_inner_size(LogicalSize { width: 100.0, height: 100.0 });

    let window_state = if config.window().restore { config.data_dir().load() } else { None };
    let (zoom_level, always_on_top, delayed_maximize) = if let Some(state) = window_state {
        log::debug!("Restoring window state: {state:?}");
        let WindowState { height, width, x, y, fullscreen, zoom_level, always_on_top, maximized } =
            state;
        builder = builder
            .with_inner_size(LogicalSize { width, height })
            .with_position(LogicalPosition { x, y });
        if fullscreen {
            builder = builder.with_fullscreen(Some(Fullscreen::Borderless(None)));
        } else if maximized {
            builder = builder.with_maximized(true);
        }
        (zoom_level, always_on_top, None)
    } else {
        let size = config.window().default_size;
        let delayed = match (size.width, size.height) {
            (WindowLength::Fixed(w), WindowLength::Fixed(h)) => {
                let size = LogicalSize { width: w.get() as f64, height: h.get() as f64 };
                builder = builder.with_inner_size(size);
                None
            }
            (WindowLength::Max, WindowLength::Max) => {
                builder = builder.with_maximized(true);
                None
            }
            (WindowLength::Fixed(width), WindowLength::Max) => {
                Some(MaximizeWindow::Vertical { width })
            }
            (WindowLength::Max, WindowLength::Fixed(height)) => {
                Some(MaximizeWindow::Horizontal { height })
            }
        };
        (ZoomLevel::default(), config.window().always_on_top, delayed)
    };

    if always_on_top {
        builder = builder.with_always_on_top(true);
    }

    match config.window().theme {
        ThemeConfig::System => {}
        ThemeConfig::Dark => builder = builder.with_theme(Some(Theme::Dark)),
        ThemeConfig::Light => builder = builder.with_theme(Some(Theme::Light)),
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        use tao::window::Icon;
        let icon = Icon::from_rgba(ICON_RGBA.into(), 32, 32).unwrap();
        builder = builder.with_window_icon(Some(icon));
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    if config.window().is_vibrant() {
        builder = builder.with_transparent(true);

        #[cfg(target_os = "windows")]
        {
            // Applying Mica effect requires a visible window, no decorations and no shadow. The decorations
            // and shadow will be enabled again after applying the effect.
            builder =
                builder.with_visible(true).with_decorations(false).with_undecorated_shadow(false);
        }
    }

    #[cfg(target_os = "macos")]
    {
        builder = builder
            .with_fullsize_content_view(true)
            .with_titlebar_transparent(true)
            .with_title_hidden(true);
    }

    // GTK does not return monitor information until the window is displayed.
    #[cfg(target_os = "linux")]
    if delayed_maximize.is_some() {
        builder = builder.with_visible(true);
    }

    let window = builder.build(event_loop)?;

    if let Some(delayed) = delayed_maximize {
        delayed.maximize(&window);
    }

    #[cfg(target_os = "windows")]
    if config.window().is_vibrant() {
        let is_dark = match config.window().theme {
            ThemeConfig::System => None,
            ThemeConfig::Dark => Some(true),
            ThemeConfig::Light => Some(false),
        };

        if let Err(err) = window_vibrancy::apply_mica(&window, is_dark) {
            log::debug!("Could not apply Mica effect. Fall back to solid window: {err}");
            let color =
                if window.theme() == Theme::Light { (255, 255, 255, 255) } else { (0, 0, 0, 255) };
            window.set_background_color(Some(color));
        }

        window.set_undecorated_shadow(true);
        window.set_decorations(true);
    }

    Ok((window, zoom_level, always_on_top))
}

fn parse_local_path_from_url(mut url: String) -> Result<InitFile, String> {
    // Custom protocol URLs are different for each platform
    //   macOS, Linux → <scheme_name>://<path>
    //   Windows → https://<scheme_name>.<path>
    #[cfg(not(target_os = "windows"))]
    const CUSTOM_PROTOCOL_URL: &str = "shiba://localhost/";
    #[cfg(target_os = "windows")]
    const CUSTOM_PROTOCOL_URL: &str = "http://shiba.localhost/";

    if !url.starts_with(CUSTOM_PROTOCOL_URL) {
        return Err(url);
    }

    url.drain(0..CUSTOM_PROTOCOL_URL.len() - 1); // shiba://localhost/foo/bar -> /foo/bar
    if url.is_empty() {
        url.push('.');
    }

    let fragment = if let Some(idx) = url.rfind('#')
        && !url[idx..].contains('/')
    {
        let frag = url[idx + 1..].to_string(); // Get hash: /a/b#foo -> foo
        url.truncate(idx); // Remove hash link: /a/b#foo -> /a/b
        Some(frag)
    } else {
        None
    };

    #[cfg(not(target_os = "windows"))]
    let path = url.into();
    #[cfg(target_os = "windows")]
    let path = url.replace('/', "\\").into();
    Ok(InitFile { path, fragment })
}

fn create_webview(window: &Window, ipc_proxy: Proxy, config: &Config) -> Result<WebView> {
    let file_drop_proxy = ipc_proxy.clone();
    let navigation_proxy = ipc_proxy.clone();
    let new_window_proxy = ipc_proxy.clone();
    let loader = Assets::new(config);

    let user_dir = config.data_dir().path().map(|dir| dir.join("WebView"));
    let id = window.id();
    log::debug!("WebView user data directory: {:?}", user_dir);
    let mut context = WebContext::new(user_dir);
    let mut builder = WebViewBuilder::new_with_web_context(&mut context);

    builder = builder
        .with_url("shiba://localhost/index.html")
        .with_ipc_handler(move |msg| {
            let message: MessageFromWindow = serde_json::from_str(msg.body()).unwrap();
            log::debug!("Message from WebView: {message:?}");
            if let Err(err) =
                ipc_proxy.send_event(Request::Emit(Event::WindowMessage { message, id }))
            {
                log::error!("Could not send user event for message from WebView: {err}");
            }
        })
        .with_drag_drop_handler(move |event| {
            if let DragDropEvent::Drop { paths, .. } = event {
                log::debug!("Files were dropped (the first one will be opened): {paths:?}",);
                // TODO: Support dropping multiple files
                if let Some(path) = paths.into_iter().next()
                    && let Err(err) =
                        file_drop_proxy.send_event(Request::Emit(Event::FileDrop { path, id }))
                {
                    log::error!("Could not send user event for file drop: {err}");
                }
            }
            true
        })
        .with_navigation_handler(move |url| {
            log::debug!("Navigating to URL: {url:?}");
            let event = match parse_local_path_from_url(url) {
                Ok(file) if &file.path == "/index.html" => return true,
                Ok(file) => Event::OpenLocalPath { file, id },
                Err(url) => Event::OpenExternalLink(url),
            };

            if let Err(e) = navigation_proxy.send_event(Request::Emit(event)) {
                log::error!("Could not send navigation event: {}", e);
            }

            false // Don't allow navigating to any external links
        })
        .with_new_window_req_handler(move |url, _| {
            log::debug!("New window request with URL: {url:?}");
            let event = match parse_local_path_from_url(url) {
                Ok(InitFile { path, fragment }) if &path == "/index.html" => {
                    Event::DuplicateWindow { fragment, id }
                }
                Ok(file) => Event::NewWindow { init_file: Some(file) },
                Err(url) => Event::OpenExternalLink(url),
            };

            if let Err(e) = new_window_proxy.send_event(Request::Emit(event)) {
                log::error!("Could not send new window event: {}", e);
            }

            NewWindowResponse::Deny
        })
        .with_custom_protocol("shiba".into(), move |_webview_id, request| {
            let uri = request.uri();
            log::debug!("Handling custom protocol: {:?}", uri);
            let path = uri.path();
            let (content, mime) = loader.load(path);
            let (body, status) =
                if let Some(content) = content { (content, 200) } else { (vec![].into(), 404) };
            // The header and status are never invalid so `.unwrap()` call never panics
            Response::builder().status(status).header(CONTENT_TYPE, mime).body(body).unwrap()
        })
        .with_focused(true)
        .with_devtools(cfg!(any(debug_assertions, feature = "devtools")));

    #[cfg(target_os = "windows")]
    {
        use wry::Theme;
        builder = builder.with_browser_accelerator_keys(false);
        let theme = match config.window().theme {
            ThemeConfig::System => Theme::Auto,
            ThemeConfig::Dark => Theme::Dark,
            ThemeConfig::Light => Theme::Light,
        };
        builder = builder.with_theme(theme);
    }

    if config.window().is_vibrant() {
        builder = builder.with_transparent(true);
    } else if window.theme() == Theme::Dark {
        // Avoid flicking window with white screen while loading webview
        builder = builder.with_background_color((0, 0, 0, 255));
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let webview = builder.build(window)?;
    #[cfg(target_os = "linux")]
    let webview = builder.build_gtk(window.default_vbox().unwrap())?;

    #[cfg(target_os = "macos")]
    if config.window().is_vibrant() {
        use window_vibrancy::{NSVisualEffectMaterial, apply_vibrancy};
        // This function must be called after the webview is inserted to the window. So this call cannot
        // be moved to `create_window` function.
        apply_vibrancy(window, NSVisualEffectMaterial::Sidebar, None, None)?;
    }

    #[cfg(any(debug_assertions, feature = "devtools"))]
    if config.debug() {
        webview.open_devtools(); // This method is defined in debug build only
        log::debug!("Opened DevTools for debugging");
    }

    Ok(webview)
}

pub struct WebViewWindow {
    webview: WebView,
    window: Window,
    zoom_level: ZoomLevel,
    always_on_top: bool,
    menu: WindowMenu,
    is_vibrant: bool,
}

impl WebViewWindow {
    pub fn new(
        config: &Config,
        event_loop: &EventLoop,
        proxy: Proxy,
        mut menu: WindowMenu,
    ) -> Result<Self> {
        let (window, zoom_level, always_on_top) = create_window(event_loop, config)?;

        if config.window().menu_bar != menu.is_visible() {
            menu.toggle(&window)?;
        }

        let webview = create_webview(&window, proxy, config)?;
        log::debug!("WebView was created successfully");

        let zoom_factor = zoom_level.factor();
        if zoom_factor != 1.0 {
            webview.zoom(zoom_factor)?;
            log::debug!("Zoom factor was set to {}", zoom_factor);
        }

        let is_vibrant = config.window().is_vibrant();
        Ok(Self { webview, window, zoom_level, always_on_top, menu, is_vibrant })
    }
}

impl RendererWindow for WebViewWindow {
    type Id = WindowId;

    fn send_message(&self, message: MessageToWindow) -> Result<()> {
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

    fn state(&self) -> Option<WindowState> {
        let scale = self.window.scale_factor();
        let LogicalPosition { x, y } = match self.window.outer_position() {
            Ok(pos) => pos.to_logical(scale),
            Err(err) => {
                log::debug!("Could not get window position for window state: {}", err);
                return None;
            }
        };

        #[cfg(not(target_os = "macos"))]
        let LogicalSize { width, height } = self.window.inner_size().to_logical(scale);
        #[cfg(target_os = "macos")]
        let (width, height) = {
            // The `inner_size` method does not work on macOS because it returns the size when the window was created
            // even if it is resized afterwards. We use the size of WebView frame instead because it is the same as the
            // window inner size in case of this application.
            use wry::WebViewExtMacOS;
            let size = self.webview.webview().frame().size;
            (size.width, size.height)
        };

        let fullscreen = self.window.fullscreen().is_some();
        let zoom_level = self.zoom_level;
        let always_on_top = self.always_on_top;
        let maximized = self.window.is_maximized();
        Some(WindowState { width, height, x, y, fullscreen, zoom_level, always_on_top, maximized })
    }

    fn show(&self) {
        self.window.set_visible(true);
    }

    fn hide(&self) {
        self.window.set_visible(false);
    }

    fn print(&self) -> Result<()> {
        Ok(self.webview.print()?)
    }

    fn zoom(&mut self, level: ZoomLevel) -> Result<()> {
        self.webview.zoom(level.factor())?;
        self.zoom_level = level;
        Ok(())
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

    fn maximize(&mut self, maximized: bool) {
        self.window.set_maximized(maximized);
    }

    fn is_minimized(&self) -> bool {
        self.window.is_minimized()
    }

    fn minimize(&mut self, minimized: bool) {
        self.window.set_minimized(minimized);
    }

    fn appearance(&self) -> WindowAppearance {
        WindowAppearance {
            title: cfg!(not(target_os = "macos")),
            vibrancy: self.is_vibrant,
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
    fn save_memory(&mut self, is_low: bool) -> Result<()> {
        let level = if is_low { MemoryUsageLevel::Low } else { MemoryUsageLevel::Normal };
        log::debug!("Memory usage level is set to {level:?} due to is_low={is_low}");
        self.webview.set_memory_usage_level(level)?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    fn save_memory(&mut self, _minimized: bool) -> Result<()> {
        Ok(())
    }

    fn delete_cache(&mut self) -> Result<()> {
        let cookies = self.webview.cookies()?;
        log::debug!("Deleting {} cookies", cookies.len());
        for cookie in cookies {
            self.webview.delete_cookie(&cookie)?;
        }
        Ok(())
    }

    fn handles(&self) -> WindowHandles<'_> {
        WindowHandles::new(&self.window)
    }

    fn focus(&self) {
        self.window.set_focus();
    }

    fn id(&self) -> Self::Id {
        self.window.id()
    }
}
