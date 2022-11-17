mod app;
mod assets;
mod cli;
mod config;
mod dialog;
mod markdown;
mod opener;
mod persistent;
mod renderer;
mod search;
mod watcher;
mod webview;

use crate::webview::Wry;
use anyhow::{Error, Result};
use app::{App, AppControl};
pub use cli::Options;
use notify::RecommendedWatcher;
use opener::SystemOpener;
use rfd::FileDialog;
use wry::application::event::{Event, StartCause, WindowEvent};
use wry::application::event_loop::{ControlFlow, EventLoop};

fn log_causes(err: Error) {
    for err in err.chain() {
        log::error!("  Caused by: {}", err);
    }
}

pub fn run(options: Options) -> Result<()> {
    let event_loop = EventLoop::with_user_event();
    let mut app =
        App::<Wry, SystemOpener, RecommendedWatcher, FileDialog>::new(options, &event_loop)?;

    event_loop.run(move |event, _, control_flow| {
        let exit = match event {
            Event::NewEvents(StartCause::Init) => {
                log::debug!("Application has started");
                false
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                log::debug!("Closing window was requested");
                true
            }
            Event::UserEvent(event) => {
                log::debug!("Handling user event {:?}", event);
                match app.handle_user_event(event) {
                    Ok(AppControl::Exit) => *control_flow = ControlFlow::Exit,
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Could not handle user event");
                        log_causes(err);
                    }
                }
                false
            }
            Event::MenuEvent { menu_id, .. } => match app.handle_menu_event(menu_id) {
                Ok(AppControl::Exit) => true,
                Ok(_) => false,
                Err(err) => {
                    log::error!("Could not handle menu event: {}", err);
                    false
                }
            },
            _ => false,
        };

        if exit {
            if let Err(err) = app.handle_exit() {
                log::error!("Could not handle application exit correctly");
                log_causes(err);
            }
            *control_flow = ControlFlow::Exit;
        } else {
            *control_flow = ControlFlow::Wait;
        }
    });
}
