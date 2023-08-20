#![warn(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]

mod app;
mod assets;
mod cli;
mod config;
mod dialog;
mod markdown;
mod opener;
mod persistent;
mod renderer;
mod watcher;
#[cfg(windows)]
mod windows;
mod wry;

pub use crate::cli::Options;
#[cfg(feature = "__bench")]
pub use crate::markdown::{MarkdownContent, MarkdownParser};
#[cfg(feature = "__bench")]
pub use crate::renderer::RawMessageWriter;
#[cfg(windows)]
pub use windows::WindowsConsole;

use crate::app::Shiba;
use crate::opener::SystemOpener;
use crate::renderer::EventLoop;
use crate::watcher::NopWatcher;
use crate::wry::{WryEventLoop, WryRenderer};
use anyhow::Result;
use notify::RecommendedWatcher;
use rfd::FileDialog;

pub fn run(options: Options) -> Result<()> {
    let event_loop = WryEventLoop::with_user_event();
    if options.watch {
        let app = Shiba::<WryRenderer, SystemOpener, RecommendedWatcher, FileDialog>::new(
            options,
            &event_loop,
        )?;
        event_loop.start(app)
    } else {
        let app =
            Shiba::<WryRenderer, SystemOpener, NopWatcher, FileDialog>::new(options, &event_loop)?;
        event_loop.start(app)
    }
}
