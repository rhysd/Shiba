#[cfg(target_os = "macos")]
use crate::assets::set_app_icon_to_dock;
use crate::config::Config;
use crate::renderer::{Event as RendererEvent, EventHandler, EventSender, Renderer, RenderingFlow};
use crate::wry::menu::Menu;
use crate::wry::webview::{EventLoop, WebViewWindow};
use anyhow::Result;
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
#[cfg(target_os = "windows")]
use tao::platform::windows::EventLoopBuilderExtWindows as _;

pub struct Wry {
    event_loop: EventLoop,
    menu: Menu,
}

impl EventSender for EventLoopProxy<RendererEvent> {
    fn send(&self, event: RendererEvent) {
        if let Err(err) = self.send_event(event) {
            log::error!("Could not send user event for message from WebView: {}", err);
        }
    }
}

impl Renderer for Wry {
    type EventSender = EventLoopProxy<RendererEvent>;
    type Window = WebViewWindow;

    fn new() -> Result<Self> {
        // `EventLoopBuilder::with_app_id` on Linux is not usable because it can cause SEGV.
        // See https://github.com/tauri-apps/tao/issues/1186

        let menu = Menu::default();

        #[cfg(not(target_os = "windows"))]
        let event_loop = EventLoopBuilder::with_user_event().build();
        #[cfg(target_os = "windows")]
        let event_loop = {
            let translator = menu.accel_translator();
            EventLoopBuilder::with_user_event()
                // Safety: `MSG` pointer passed from `EventLoop` is valid.
                .with_msg_hook(move |msg| unsafe { translator.translate(msg as *const _) })
                .build()
        };

        menu.create(event_loop.create_proxy())?;
        Ok(Self { event_loop, menu })
    }

    fn create_sender(&self) -> Self::EventSender {
        self.event_loop.create_proxy()
    }

    fn create_window(&self, config: &Config) -> Result<Self::Window> {
        WebViewWindow::new(config, &self.event_loop, self.menu.window_menu())
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
                        handler.on_event(RendererEvent::Minimized(is_minimized))
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
