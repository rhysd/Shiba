// Do not pop up cmd.exe window when clicking .exe on Windows. This also disables any console outputs
// so logger output will not work.
// https://learn.microsoft.com/en-us/cpp/build/reference/subsystem-specify-subsystem
#![cfg_attr(all(windows, not(debug_assertions), not(__bench)), windows_subsystem = "windows")]

use anyhow::Result;
use log::LevelFilter;
use shiba_preview::{run, Options};
use std::env;

fn main() -> Result<()> {
    let Some(options) = Options::from_args(env::args().skip(1))? else { return Ok(()) };
    let level = if options.debug { LevelFilter::Debug } else { LevelFilter::Info };
    env_logger::builder()
        .filter_level(level)
        .format_timestamp(None)
        .filter_module("html5ever", LevelFilter::Off)
        .init();
    run(options)
}
