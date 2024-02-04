use crate::config::Config;
use crate::renderer::{Event as AppEvent, EventHandler, EventSender, Rendering, RenderingFlow};
use crate::wry::menu::{Menu, MenuEvents};
use crate::wry::webview::{EventLoop, WebViewRenderer};
use anyhow::Result;
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};

pub struct Wry {
    event_loop: EventLoop,
    menu_events: MenuEvents,
    #[cfg(target_os = "windows")]
    menu: Menu, // On Windows, Menu instance needs to be created before creating the event loop
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
        let event_loop = EventLoopBuilder::with_user_event().build();
        let menu_events = MenuEvents::new();
        Ok(Self { event_loop, menu_events })
    }

    #[cfg(target_os = "windows")]
    fn new() -> Result<Self> {
        use tao::platform::windows::EventLoopBuilderExtWindows;
        use windows_sys::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, MSG};

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
                .build()
        };

        Ok(Self { event_loop, menu_events, menu })
    }

    fn create_sender(&self) -> Self::EventSender {
        self.event_loop.create_proxy()
    }

    fn create_renderer(&mut self, config: &Config) -> Result<Self::Renderer> {
        #[cfg(not(target_os = "windows"))]
        let menu = Menu::new(&mut self.menu_events)?;
        #[cfg(target_os = "windows")]
        let menu = self.menu.clone();
        WebViewRenderer::new(config, &self.event_loop, menu)
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
                    RenderingFlow::Continue
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    log::debug!("Closing window was requested");
                    RenderingFlow::Exit
                }
                Event::UserEvent(event) => handler.handle_event(event).unwrap_or_else(|err| {
                    handler.handle_error(err.context("Could not handle user event"))
                }),
                Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                    let next_minimized = size.height == 0 || size.width == 0;
                    if next_minimized != is_minimized {
                        is_minimized = next_minimized;
                        log::debug!("Minimized state changed: {is_minimized}");
                        handler.handle_minimized(is_minimized);
                    }
                    RenderingFlow::Continue
                }
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

            *control = match flow {
                RenderingFlow::Continue => ControlFlow::Wait,
                RenderingFlow::Exit => match handler.handle_exit() {
                    Ok(()) => ControlFlow::Exit,
                    Err(err) => {
                        handler.handle_error(err.context("Could not handle application exit"));
                        ControlFlow::ExitWithCode(1)
                    }
                },
            };
        })
    }
}
