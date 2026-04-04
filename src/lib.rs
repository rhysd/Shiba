#![warn(
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::undocumented_unsafe_blocks
)]
#![allow(clippy::uninlined_format_args)]

mod assets;
mod cli;
mod config;
mod dialog;
mod history;
#[cfg(target_os = "macos")]
mod macos;
mod markdown;
mod opener;
mod persistent;
mod preview;
mod process_singleton;
mod renderer;
#[cfg(feature = "__sanity")]
mod sanity;
mod shiba;
#[cfg(test)]
mod test;
mod watcher;
mod window;
#[cfg(target_os = "windows")]
mod windows;
mod wry;

pub use cli::{Options, Parsed};
#[cfg(target_os = "windows")]
pub use windows::WindowsConsole;

#[cfg(feature = "__bench")]
pub mod bench {
    pub use super::config::SearchMatcher;
    pub use super::history::History;
    pub use super::markdown::{
        DisplayText, MarkdownContent, MarkdownParser, modified_offset, modified_offset_scalar,
    };
    pub use super::renderer::RawMessageWriter;
}

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
