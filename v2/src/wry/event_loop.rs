use crate::renderer::{EventChannel, EventLoop, EventLoopFlow, EventLoopHandler, UserEvent};
use anyhow::Error;
use wry::application::event::{Event, StartCause, WindowEvent};
use wry::application::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};

pub type WryEventLoop = wry::application::event_loop::EventLoop<UserEvent>;

impl EventChannel for EventLoopProxy<UserEvent> {
    fn send_event(&self, event: UserEvent) {
        if let Err(err) = self.send_event(event) {
            log::error!("Could not send user event for message from WebView: {}", err);
        }
    }
}

impl EventLoop for WryEventLoop {
    type Channel = EventLoopProxy<UserEvent>;

    fn new() -> Self {
        EventLoopBuilder::with_user_event().build()
    }

    fn create_channel(&self) -> Self::Channel {
        self.create_proxy()
    }

    fn start<H>(self, mut handler: H) -> !
    where
        H: EventLoopHandler + 'static,
    {
        self.run(move |event, _, control| {
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
