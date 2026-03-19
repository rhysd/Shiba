#[cfg(target_os = "macos")]
use crate::assets::set_app_icon_to_dock;
use crate::config::Config;
use crate::renderer::{EventHandler, Renderer, RendererHandle, RenderingFlow};
use crate::wry::menu::Menu;
use crate::wry::types::{Event as RendererEvent, Proxy, Request};
use crate::wry::webview::WebViewWindow;
use anyhow::Result;
use std::rc::Rc;
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::EventLoop;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
#[cfg(target_os = "windows")]
use tao::platform::windows::EventLoopBuilderExtWindows as _;
use tao::window::WindowId;

pub struct Wry {
    event_loop: EventLoop<Request>,
    menu: Menu,
    config: Rc<Config>,
}

impl RendererHandle for Proxy {
    type WindowId = WindowId;

    fn send(&self, event: RendererEvent) {
        if let Err(err) = self.send_event(Request::Event(event)) {
            log::error!("Could not send user event for message from WebView: {}", err);
        }
    }

    fn create_window(&self) {
        if let Err(err) = self.send_event(Request::CreateWindow) {
            log::error!("Could not send window creation request: {}", err);
        }
    }
}

impl Renderer for Wry {
    type Handle = Proxy;
    type WindowId = WindowId;
    type Window = WebViewWindow;

    fn new(config: Rc<Config>) -> Result<Self> {
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
        Ok(Self { event_loop, menu, config })
    }

    fn create_handle(&self) -> Self::Handle {
        self.event_loop.create_proxy()
    }

    fn start<H>(self, mut handler: H) -> !
    where
        H: EventHandler<Window = Self::Window, WindowId = Self::WindowId> + 'static,
    {
        let proxy = self.event_loop.create_proxy();
        let mut is_minimized = false;
        self.event_loop.run(move |event, event_loop, control| {
            let flow = match event {
                Event::NewEvents(StartCause::Init) => {
                    log::debug!("Application has started");

                    // App icon should be set to dock at `applicationDidFinishLaunching:` and `StartCause::Init` event
                    // is emitted on the hook.
                    #[cfg(target_os = "macos")]
                    set_app_icon_to_dock();

                    RenderingFlow::Continue
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, window_id, .. } => {
                    log::debug!("Closing window was requested: {window_id:?}");
                    handler.on_window_closed(window_id)
                }
                Event::UserEvent(request) => match request {
                    Request::Event(event) => handler.on_event(event),
                    Request::CreateWindow => {
                        let created = WebViewWindow::new(
                            &self.config,
                            event_loop,
                            proxy.clone(),
                            self.menu.window_menu(),
                        );
                        match created {
                            Ok(window) => handler.on_window_created(window),
                            Err(err) => handler.on_event(RendererEvent::Error(err)),
                        }
                    }
                },
                Event::WindowEvent { event: WindowEvent::Resized(size), window_id, .. } => {
                    let next_minimized = size.height == 0 || size.width == 0;
                    if next_minimized != is_minimized {
                        is_minimized = next_minimized;
                        log::debug!("Minimized state changed for {window_id:?}: {is_minimized}");
                        handler.on_window_minimized(is_minimized, window_id)
                    } else {
                        RenderingFlow::Continue
                    }
                }
                Event::WindowEvent { event: WindowEvent::Focused(true), window_id, .. } => {
                    handler.on_window_focused(window_id)
                }
                _ => RenderingFlow::Continue,
            };

            *control = match flow {
                RenderingFlow::Continue => ControlFlow::Wait,
                RenderingFlow::Exit(code) => ControlFlow::ExitWithCode(code),
            };
        })
    }
}
