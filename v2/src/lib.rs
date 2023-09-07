#![warn(
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::undocumented_unsafe_blocks
)]

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

use crate::opener::SystemOpener;
use crate::renderer::EventLoop;
use crate::watcher::{NopWatcher, SystemWatcher};
use crate::wry::{WryEventLoop, WryRenderer};
use anyhow::Result;
use rfd::FileDialog;

pub fn run(options: Options) -> Result<()> {
    type Shiba<W> = app::Shiba<WryRenderer, SystemOpener, W, FileDialog>;
    let event_loop = WryEventLoop::new();
    if options.watch {
        let app = Shiba::<SystemWatcher>::new(options, &event_loop)?;
        event_loop.start(app)
    } else {
        let app = Shiba::<NopWatcher>::new(options, &event_loop)?;
        event_loop.start(app)
    }
}
