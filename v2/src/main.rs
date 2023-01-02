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
