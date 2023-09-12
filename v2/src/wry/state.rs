use crate::config::Config;
use crate::renderer::{EventHandler, RendererFlow, RendererState, UserEvent, UserEventSender};
use crate::wry::menu::Menu;
use crate::wry::webview::{EventLoop, WebViewRenderer};
use anyhow::{Error, Result};
use wry::application::event::{Event, StartCause, WindowEvent};
use wry::application::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};

pub struct Wry {
    event_loop: EventLoop,
    menu: Menu,
}

impl UserEventSender for EventLoopProxy<UserEvent> {
    fn send(&self, event: UserEvent) {
        if let Err(err) = self.send_event(event) {
            log::error!("Could not send user event for message from WebView: {}", err);
        }
    }
}

impl RendererState for Wry {
    type UserEventSender = EventLoopProxy<UserEvent>;
    type Renderer = WebViewRenderer;

    #[cfg(not(windows))]
    fn new() -> Self {
        Self { event_loop: EventLoopBuilder::with_user_event().build(), menu: Menu::new() }
    }

    #[cfg(windows)]
    fn new() -> Self {
        use windows_sys::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, MSG};
        use wry::application::platform::windows::EventLoopBuilderExtWindows;

        let menu = Menu::new();
        let event_loop = {
            let menu = menu.menu_bar().clone();
            EventLoopBuilder::with_user_event()
                .with_msg_hook(move |msg| {
                    let msg = msg as *const MSG;
                    let haccel = menu.haccel();
                    // SAFETY: Win32 API usage is always unsafe
                    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translateacceleratorw
                    let translated = unsafe { TranslateAcceleratorW((*msg).hwnd, haccel, msg) };
                    translated != 0
                })
                .build()
        };

        Self { event_loop, menu }
    }

    fn create_sender(&self) -> Self::UserEventSender {
        self.event_loop.create_proxy()
    }

    fn create_renderer(&mut self, config: &Config) -> Result<Self::Renderer> {
        let renderer = WebViewRenderer::new(config, &self.event_loop)?;
        self.menu.setup(renderer.window())?;
        Ok(renderer)
    }

    fn start<H>(self, mut handler: H) -> !
    where
        H: EventHandler + 'static,
    {
        self.event_loop.run(move |event, _, control| {
            fn log_causes(err: Error) {
                for err in err.chain() {
                    log::error!("  Caused by: {}", err);
                }
            }

            let flow = match event {
                Event::NewEvents(StartCause::Init) => {
                    log::debug!("Application has started");
                    RendererFlow::Continue
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    log::debug!("Closing window was requested");
                    RendererFlow::Break
                }
                Event::UserEvent(event) => handler.handle_user_event(event).unwrap_or_else(|err| {
                    log::error!("Could not handle user event");
                    log_causes(err);
                    RendererFlow::Continue
                }),
                _ => handler.handle_menu_event(&self.menu).unwrap_or_else(|err| {
                    log::error!("Could not handle menu event");
                    log_causes(err);
                    RendererFlow::Continue
                }),
            };

            *control = match flow {
                RendererFlow::Continue => ControlFlow::Wait,
                RendererFlow::Break => match handler.handle_exit() {
                    Ok(_) => ControlFlow::Exit,
                    Err(err) => {
                        log::error!("Could not handle application exit correctly");
                        log_causes(err);
                        ControlFlow::ExitWithCode(1)
                    }
                },
            };
        })
    }
}
