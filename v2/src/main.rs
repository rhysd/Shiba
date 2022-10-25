use anyhow::Result;
use shiba_preview::{run, Options};
use std::env;

fn main() -> Result<()> {
    let debug = env::var("SHIBA_DEBUG").is_ok();
    let level = if debug { log::LevelFilter::Debug } else { log::LevelFilter::Info };

    env_logger::builder().filter_level(level).init();

    if let Some(mut options) = Options::from_args(env::args().skip(1))? {
        options.debug = debug;
        run(options)?;
    }

    Ok(())
}
