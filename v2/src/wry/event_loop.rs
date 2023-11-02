use crate::config::Config;
use crate::renderer::{
    EventHandler, Rendering, RenderingFlow, Theme as RendererTheme, UserEvent, UserEventSender,
};
use crate::wry::menu::{Menu, MenuEvents};
use crate::wry::webview::{EventLoop, WebViewRenderer};
use anyhow::{Error, Result};
use wry::application::event::{Event, StartCause, WindowEvent};
use wry::application::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use wry::application::window::Theme;

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

    #[cfg(not(windows))]
    fn new() -> Result<Self> {
        let event_loop = EventLoopBuilder::with_user_event().build();
        let menu_events = MenuEvents::new();
        Ok(Self { event_loop, menu_events })
    }

    #[cfg(windows)]
    fn new() -> Result<Self> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, MSG};
        use wry::application::platform::windows::EventLoopBuilderExtWindows;

        let mut menu_events = MenuEvents::new();
        let menu = Menu::new(&mut menu_events)?;
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
                    RenderingFlow::Continue
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    log::debug!("Closing window was requested");
                    RenderingFlow::Exit
                }
                Event::WindowEvent {
                    event: WindowEvent::ThemeChanged(theme @ (Theme::Light | Theme::Dark)),
                    ..
                } => {
                    log::debug!("Window theme was changed: {:?}", theme);
                    let theme = match theme {
                        Theme::Light => RendererTheme::Light,
                        Theme::Dark => RendererTheme::Dark,
                        _ => unreachable!(),
                    };
                    handler.handle_theme_changed(theme).unwrap_or_else(|err| {
                        log::error!("Could not handle theme change: {:?}", theme);
                        log_causes(err);
                        RenderingFlow::Continue
                    })
                }
                Event::UserEvent(event) => handler.handle_user_event(event).unwrap_or_else(|err| {
                    log::error!("Could not handle user event");
                    log_causes(err);
                    RenderingFlow::Continue
                }),
                _ => self
                    .menu_events
                    .try_receive()
                    .and_then(|item| match item {
                        Some(item) => handler.handle_menu_event(item),
                        None => Ok(RenderingFlow::Continue),
                    })
                    .unwrap_or_else(|err| {
                        log::error!("Could not handle menu event");
                        log_causes(err);
                        RenderingFlow::Continue
                    }),
            };

            *control = match flow {
                RenderingFlow::Continue => ControlFlow::Wait,
                RenderingFlow::Exit => match handler.handle_exit() {
                    Ok(()) => ControlFlow::Exit,
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
