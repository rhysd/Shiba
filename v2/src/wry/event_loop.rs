use crate::config::Config;
use crate::renderer::{EventHandler, Rendering, RenderingFlow, UserEvent, UserEventSender};
use crate::wry::menu::{Menu, MenuEvents};
use crate::wry::webview::{EventLoop, WebViewRenderer};
use anyhow::Result;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};

pub struct Wry {
    event_loop: EventLoop,
    menu_events: MenuEvents,
    #[cfg(target_os = "windows")]
    menu: Menu, // On Windows, Menu instance needs to be created before creating the event loop
}

impl UserEventSender for EventLoopProxy<UserEvent> {
    fn send(&self, event: UserEvent) {
        if let Err(err) = self.send_event(event) {
            log::error!("Could not send user event for message from WebView: {}", err);
        }
    }
}

impl Rendering for Wry {
    type UserEventSender = EventLoopProxy<UserEvent>;
    type Renderer = WebViewRenderer;

    #[cfg(not(target_os = "windows"))]
    fn new() -> Result<Self> {
        let event_loop = EventLoopBuilder::with_user_event().build()?;
        let menu_events = MenuEvents::new();
        Ok(Self { event_loop, menu_events })
    }

    #[cfg(target_os = "windows")]
    fn new() -> Result<Self> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, MSG};
        use winit::platform::windows::EventLoopBuilderExtWindows;

        let mut menu_events = MenuEvents::new();
        let menu = Menu::new(&mut menu_events)?;
        let event_loop = {
            let menu = menu.menu_bar().clone();
            EventLoopBuilder::with_user_event()
                .with_msg_hook(move |msg| {
                    let msg = msg as *const MSG;
                    let haccel = menu.haccel();
                    // SAFETY: `msg` pointer was given by `EventLoopBuilder::with_user_event` which internally receives
                    // events via message loop. `haccel` is validated by muda's API.
                    // Ref: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translateacceleratorw
                    let translated = unsafe { TranslateAcceleratorW((*msg).hwnd, haccel, msg) };
                    translated != 0
                })
                .build()?
        };

        Ok(Self { event_loop, menu_events, menu })
    }

    fn create_sender(&self) -> Self::UserEventSender {
        self.event_loop.create_proxy()
    }

    fn create_renderer(&mut self, config: &Config) -> Result<Self::Renderer> {
        #[cfg(not(target_os = "windows"))]
        let menu = Menu::new(&mut self.menu_events)?;
        #[cfg(target_os = "windows")]
        let menu = self.menu.clone();
        WebViewRenderer::new(config, &self.event_loop, menu)
    }

    fn run<H: EventHandler>(self, mut handler: H) -> Result<()> {
        self.event_loop.run(|event, target| {
            let flow = match event {
                Event::NewEvents(StartCause::Init) => {
                    log::debug!("Application has started");
                    RenderingFlow::Continue
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    log::debug!("Closing window was requested");
                    RenderingFlow::Close
                }
                Event::UserEvent(event) => handler.handle_user_event(event).unwrap_or_else(|err| {
                    handler.handle_error(err.context("Could not handle user event"))
                }),
                _ => self
                    .menu_events
                    .try_receive()
                    .and_then(|item| match item {
                        Some(item) => handler.handle_menu_event(item),
                        None => Ok(RenderingFlow::Continue),
                    })
                    .unwrap_or_else(|err| {
                        handler.handle_error(err.context("Could not handle menu event"))
                    }),
            };

            match flow {
                RenderingFlow::Continue => target.set_control_flow(ControlFlow::Wait),
                RenderingFlow::Close => {
                    if let Err(err) = handler.handle_close() {
                        handler.handle_error(err.context("Could not handle application exit"));
                    }
                    target.exit();
                }
            };
        })?;
        handler.handle_exit()
    }
}
