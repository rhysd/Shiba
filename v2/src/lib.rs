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

use crate::dialog::SystemDialog;
use crate::opener::SystemOpener;
use crate::renderer::Rendering as _;
use crate::watcher::{NopWatcher, SystemWatcher};
use crate::wry::Wry;
use anyhow::Result;

pub fn run(options: Options) -> Result<()> {
    type Shiba<W> = app::Shiba<Wry, SystemOpener, W, SystemDialog>;
    let mut wry = Wry::new();
    if options.watch {
        let app = Shiba::<SystemWatcher>::new(options, &mut wry)?;
        wry.start(app)
    } else {
        let app = Shiba::<NopWatcher>::new(options, &mut wry)?;
        wry.start(app)
    }
}
