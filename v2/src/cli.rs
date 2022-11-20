use crate::config::WindowTheme;
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
    pub theme: Option<WindowTheme>,
    pub gen_config_file: bool,
}

impl Options {
    pub fn from_args(iter: impl Iterator<Item = String>) -> Result<Option<Self>> {
        let mut opts = GetOpts::new();
        opts.optflag("h", "help", "print this help");
        opts.optopt("t", "theme", r#"window theme ("dark", "light" or "system")"#, "THEME");
        opts.optflag(
            "",
            "generate-config-file",
            "generate default config file at the default config directory. this overwrites an existing file",
        );

        let matches = opts.parse(iter)?;
        if matches.opt_present("h") {
            println!("{}", opts.usage("Usage: shiba [option] [FILE] [DIR...]"));
            return Ok(None);
        }
        let theme = match matches.opt_str("t") {
            Some(theme) => match theme.as_str() {
                "dark" => Some(WindowTheme::Dark),
                "light" => Some(WindowTheme::Light),
                "system" => Some(WindowTheme::System),
                _ => anyhow::bail!(
                    r#"Value for --theme must be one of "dark", "light" or "system" but got {:}"#,
                    theme,
                ),
            },
            None => None,
        };
        let gen_config_file = matches.opt_present("generate-config-file");

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

        Ok(Some(Self { debug: false, init_file, watch_dirs, theme, gen_config_file }))
    }
}
