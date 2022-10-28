use anyhow::Result;
use getopts::Options as GetOpts;
use std::env;
use std::path::PathBuf;

#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Options {
    pub debug: bool,
    pub init_file: Option<PathBuf>,
    pub watch_dirs: Vec<PathBuf>,
}

impl Options {
    pub fn from_args(iter: impl Iterator<Item = String>) -> Result<Option<Self>> {
        let mut opts = GetOpts::new();
        opts.optflag("h", "help", "print this help");
        let matches = opts.parse(iter)?;
        if matches.opt_present("h") {
            println!("{}", opts.usage("Usage: shiba [option] [FILE] [DIR...]"));
            return Ok(None);
        }

        let mut init_file = None;
        let mut watch_dirs = vec![];
        for arg in matches.free.into_iter() {
            let path = PathBuf::from(arg);
            if path.is_dir() {
                watch_dirs.push(path.canonicalize()?);
            } else if let Some(f) = init_file {
                anyhow::bail!("Only single file path can be specified ({:?} v.s. {:?})", f, path);
            } else if path.exists() {
                init_file = Some(path.canonicalize()?);
            } else if path.is_absolute() {
                init_file = Some(path);
            } else {
                init_file = Some(env::current_dir()?.join(path));
            }
        }

        Ok(Some(Self { debug: false, init_file, watch_dirs }))
    }
}
