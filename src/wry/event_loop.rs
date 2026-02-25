#[cfg(target_os = "macos")]
use crate::assets::set_app_icon_to_dock;
use crate::config::Config;
use crate::renderer::{Event as AppEvent, EventHandler, EventSender, Rendering, RenderingFlow};
use crate::wry::menu::Menu;
use crate::wry::webview::{EventLoop, WebViewRenderer};
use anyhow::Result;
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};

pub struct Wry {
    event_loop: EventLoop,
    menu: Menu,
}

impl EventSender for EventLoopProxy<AppEvent> {
    fn send(&self, event: AppEvent) {
        if let Err(err) = self.send_event(event) {
            log::error!("Could not send user event for message from WebView: {}", err);
        }
    }
}

impl Rendering for Wry {
    type EventSender = EventLoopProxy<AppEvent>;
    type Renderer = WebViewRenderer;

    #[cfg(not(target_os = "windows"))]
    fn new() -> Result<Self> {
        // `EventLoopBuilder::with_app_id` on Linux is not usable because it can cause SEGV.
        // See https://github.com/tauri-apps/tao/issues/1186

        let event_loop = EventLoopBuilder::with_user_event().build();
        let menu = Menu::default();
        menu.create(event_loop.create_proxy())?;
        Ok(Self { event_loop, menu })
    }

    #[cfg(target_os = "windows")]
    fn new() -> Result<Self> {
        use std::ffi::c_void;
        use tao::platform::windows::EventLoopBuilderExtWindows;
        use windows::Win32::UI::WindowsAndMessaging::{HACCEL, MSG, TranslateAcceleratorW};

        let menu = Menu::default();
        let event_loop = {
            let menu = menu.menu_bar().clone();
            EventLoopBuilder::with_user_event()
                .with_msg_hook(move |msg| {
                    let msg = msg as *const MSG;
                    // Note: windows-sys v0.52 (depended by muda) returns `isize` but windows v0.58 requires `*mut c_void`
                    let haccel = HACCEL(menu.haccel() as *mut c_void);
                    // SAFETY: `msg` pointer was given by `EventLoopBuilder::with_msg_hook` which internally receives
                    // events via message loop. `haccel` is validated by muda's API.
                    // Ref: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translateacceleratorw
                    let translated = unsafe { TranslateAcceleratorW((*msg).hwnd, haccel, msg) };
                    translated != 0
                })
                .build()
        };
        menu.create(event_loop.create_proxy())?;
        Ok(Self { event_loop, menu })
    }

    fn create_sender(&self) -> Self::EventSender {
        self.event_loop.create_proxy()
    }

    fn create_renderer(&self, config: &Config) -> Result<Self::Renderer> {
        WebViewRenderer::new(config, &self.event_loop, self.menu.clone())
    }

    fn start<H>(self, mut handler: H) -> !
    where
        H: EventHandler + 'static,
    {
        let mut is_minimized = false;
        self.event_loop.run(move |event, _, control| {
            let flow = match event {
                Event::NewEvents(StartCause::Init) => {
                    log::debug!("Application has started");

                    // App icon should be set to dock at `applicationDidFinishLaunching:` and `StartCause::Init` event
                    // is emitted on the hook.
                    #[cfg(target_os = "macos")]
                    set_app_icon_to_dock();

                    RenderingFlow::Continue
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    log::debug!("Closing window was requested");
                    RenderingFlow::Exit
                }
                Event::UserEvent(event) => handler.on_event(event),
                Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                    let next_minimized = size.height == 0 || size.width == 0;
                    if next_minimized != is_minimized {
                        is_minimized = next_minimized;
                        log::debug!("Minimized state changed: {is_minimized}");
                        handler.on_event(AppEvent::Minimized(is_minimized))
                    } else {
                        RenderingFlow::Continue
                    }
                }
                _ => RenderingFlow::Continue,
            };

            *control = match flow {
                RenderingFlow::Continue => ControlFlow::Wait,
                RenderingFlow::Exit => ControlFlow::ExitWithCode(handler.on_exit()),
            };
        })
    }
}
