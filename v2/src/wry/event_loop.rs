use crate::renderer::{EventChannel, EventLoop, EventLoopFlow, EventLoopHandler, UserEvent};
use anyhow::Error;
use wry::application::event::{Event, StartCause, WindowEvent};
use wry::application::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};

pub type InnerEventLoop = wry::application::event_loop::EventLoop<UserEvent>;

pub struct WryEventLoop {
    inner: InnerEventLoop,
    #[cfg(windows)]
    menu_bar: muda::Menu,
}

impl WryEventLoop {
    pub fn inner(&self) -> &InnerEventLoop {
        &self.inner
    }

    #[cfg(windows)]
    pub fn menu_bar(&self) -> muda::Menu {
        self.menu_bar.clone()
    }
}

impl EventChannel for EventLoopProxy<UserEvent> {
    fn send_event(&self, event: UserEvent) {
        if let Err(err) = self.send_event(event) {
            log::error!("Could not send user event for message from WebView: {}", err);
        }
    }
}

impl EventLoop for WryEventLoop {
    type Channel = EventLoopProxy<UserEvent>;

    #[cfg(not(windows))]
    fn new() -> Self {
        Self { inner: EventLoopBuilder::with_user_event().build() }
    }

    #[cfg(windows)]
    fn new() -> Self {
        use windows_sys::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, MSG};
        use wry::application::platform::windows::EventLoopBuilderExtWindows;
        let menu_bar = muda::Menu::new();
        let inner = {
            let menu_bar = menu_bar.clone();
            EventLoopBuilder::with_user_event()
                .with_msg_hook(move |msg| {
                    let msg = msg as *const MSG;
                    let haccel = menu_bar.haccel();
                    // SAFETY: Win32 API usage is always unsafe
                    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translateacceleratorw
                    let translated = unsafe { TranslateAcceleratorW((*msg).hwnd, haccel, msg) };
                    translated != 0
                })
                .build()
        };
        Self { inner, menu_bar }
    }

    fn create_channel(&self) -> Self::Channel {
        self.inner.create_proxy()
    }

    fn start<H>(self, mut handler: H) -> !
    where
        H: EventLoopHandler + 'static,
    {
        self.inner.run(move |event, _, control| {
            fn log_causes(err: Error) {
                for err in err.chain() {
                    log::error!("  Caused by: {}", err);
                }
            }

            let flow = match event {
                Event::NewEvents(StartCause::Init) => {
                    log::debug!("Application has started");
                    EventLoopFlow::Continue
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    log::debug!("Closing window was requested");
                    EventLoopFlow::Break
                }
                Event::UserEvent(event) => handler.handle_user_event(event).unwrap_or_else(|err| {
                    log::error!("Could not handle user event");
                    log_causes(err);
                    EventLoopFlow::Continue
                }),
                _ => handler.handle_menu_event().unwrap_or_else(|err| {
                    log::error!("Could not handle menu event");
                    log_causes(err);
                    EventLoopFlow::Continue
                }),
            };

            *control = match flow {
                EventLoopFlow::Continue => ControlFlow::Wait,
                EventLoopFlow::Break => match handler.handle_exit() {
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
