#![warn(
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::undocumented_unsafe_blocks
)]

mod assets;
mod cli;
mod config;
mod dialog;
mod history;
mod markdown;
mod opener;
mod persistent;
mod renderer;
#[cfg(feature = "__sanity")]
mod sanity;
mod shiba;
mod watcher;
#[cfg(target_os = "windows")]
mod windows;
mod wry;

pub use cli::{Options, Parsed};
#[cfg(feature = "__bench")]
pub use markdown::{MarkdownContent, MarkdownParser};
#[cfg(feature = "__bench")]
pub use renderer::RawMessageWriter;
#[cfg(target_os = "windows")]
pub use windows::WindowsConsole;

use anyhow::Result;
use dialog::SystemDialog;
use opener::SystemOpener;
use shiba::Shiba;
use watcher::{NopWatcher, SystemWatcher};
use wry::Wry;

pub fn run(options: Options) -> Result<()> {
    if options.watch {
        Shiba::<Wry, SystemOpener, SystemWatcher, SystemDialog>::run(options)
    } else {
        Shiba::<Wry, SystemOpener, NopWatcher, SystemDialog>::run(options)
    }
}
