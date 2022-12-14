use anyhow::Result;
use log::LevelFilter;
use shiba_preview::{run, Options};
use std::env;

fn main() -> Result<()> {
    let debug = env::var("SHIBA_DEBUG").is_ok();

    env_logger::builder()
        .filter_level(if debug { LevelFilter::Debug } else { LevelFilter::Info })
        .format_timestamp(None)
        .filter_module("html5ever", LevelFilter::Off)
        .init();

    if let Some(mut options) = Options::from_args(env::args().skip(1))? {
        options.debug = debug;
        run(options)?;
    }

    Ok(())
}
