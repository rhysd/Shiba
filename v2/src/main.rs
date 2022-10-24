use anyhow::Result;
use getopts::Options;
use shiba_preview::run;
use std::env;
use std::path::PathBuf;

fn usage(options: Options) {
    let program = env::args().next().unwrap();
    let header = format!("Usage: {} [option] FILE", program);
    println!("{}", options.usage(&header));
}

fn main() -> Result<()> {
    let debug = env::var("DEBUG").is_ok();
    let level = if debug { log::LevelFilter::Debug } else { log::LevelFilter::Info };

    env_logger::builder().filter_level(level).init();

    let mut options = Options::new();
    options.optflag("h", "help", "print this help");
    let matches = options.parse(env::args().skip(1))?;
    if matches.opt_present("h") {
        usage(options);
        return Ok(());
    }
    let path = matches.free.into_iter().next().map(PathBuf::from);

    run(path, debug)
}
