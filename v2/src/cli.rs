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
    pub watch_paths: Vec<PathBuf>,
    pub watch: bool,
    pub theme: Option<WindowTheme>,
    pub gen_config_file: bool,
    pub config_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
}

impl Options {
    #[allow(clippy::print_stdout)]
    pub fn from_args(iter: impl Iterator<Item = String>) -> Result<Option<Self>> {
        let mut opts = GetOpts::new();
        opts.optflag("h", "help", "print this help");
        opts.optopt("t", "theme", r#"window theme ("dark", "light" or "system")"#, "THEME");
        opts.optflag("", "no-watch", "disable to watch file changes");
        opts.optflag(
            "",
            "generate-config-file",
            "generate default config file at the config directory. this overwrites an existing file",
        );
        opts.optopt("", "config-dir", "custom config directory path", "PATH");
        opts.optopt("", "data-dir", "custom data directory path", "PATH");
        opts.optflag("", "debug", "enable debug features");

        let matches = opts.parse(iter)?;
        if matches.opt_present("h") {
            println!("{}", opts.usage("Usage: shiba [option] [PATH...]"));
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
        let watch = !matches.opt_present("no-watch");
        let gen_config_file = matches.opt_present("generate-config-file");
        let config_dir = matches.opt_str("config-dir").map(PathBuf::from);
        let data_dir = matches.opt_str("data-dir").map(PathBuf::from);
        let debug = matches.opt_present("debug");

        let mut init_file = None;
        let mut watch_paths = vec![];
        for arg in matches.free.into_iter() {
            let path = PathBuf::from(arg);
            let exists = path.exists();
            let path = if exists { path.canonicalize()? } else { env::current_dir()?.join(path) };
            if init_file.is_some() || path.is_dir() {
                watch_paths.push(path);
            } else {
                init_file = Some(path);
            }
        }

        Ok(Some(Self {
            debug,
            init_file,
            watch_paths,
            watch,
            theme,
            gen_config_file,
            config_dir,
            data_dir,
        }))
    }
}
