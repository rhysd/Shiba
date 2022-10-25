use anyhow::Result;
use getopts::Options as GetOpts;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Options {
    pub debug: bool,
    pub init_file: Option<PathBuf>,
}

impl Options {
    pub fn from_args(iter: impl Iterator<Item = String>) -> Result<Option<Self>> {
        let mut opts = GetOpts::new();
        opts.optflag("h", "help", "print this help");
        let matches = opts.parse(iter)?;
        if matches.opt_present("h") {
            println!("{}", opts.usage("Usage: shiba [option] FILE"));
            return Ok(None);
        }

        Ok(Some(Self {
            debug: false,
            init_file: matches.free.into_iter().next().map(PathBuf::from),
        }))
    }
}
