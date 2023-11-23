use anyhow::{Error, Result};
use once_cell::unsync::OnceCell;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeOption {
    System,
    Dark,
    Light,
}

impl FromStr for ThemeOption {
    type Err = Error;

    fn from_str(name: &str) -> Result<Self> {
        match name {
            "dark" | "Dark" => Ok(Self::Dark),
            "light" | "Light" => Ok(Self::Light),
            "system" | "System" => Ok(Self::System),
            _ => anyhow::bail!(
                r#"Value for --theme must be one of "dark", "light" or "system" but got {name:?}"#,
            ),
        }
    }
}

#[derive(Debug)]
pub enum Parsed {
    Options(Options),
    Help(&'static str),
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct Options {
    pub debug: bool,
    pub init_file: Option<PathBuf>,
    pub watch_paths: Vec<PathBuf>,
    pub watch: bool,
    pub theme: Option<ThemeOption>,
    pub gen_config_file: bool,
    pub config_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            debug: false,
            init_file: None,
            watch_paths: vec![],
            watch: true,
            theme: None,
            gen_config_file: false,
            config_dir: None,
            data_dir: None,
        }
    }
}

impl Options {
    const USAGE: &'static str = r#"Usage: shiba [OPTIONS...] [PATH...]

Shiba is a markdown preview application to be used with your favorite text
editor, designed for simplicity, performance, and keyboard-friendliness.

Options:
    -t, --theme THEME           Window theme ("dark", "light" or "system")
        --no-watch              Disable to watch file changes
        --generate-config-file  Generate the default config file overwriting an existing file
        --config-dir PATH       Change the config directory path
        --data-dir PATH         Change the application data directory path
        --debug                 Enable debug features
    -h, --help                  Print this help

Document:
    https://github.com/rhysd/Shiba/v2/README.md
"#;

    pub fn parse(args: impl IntoIterator<Item = impl Into<OsString>>) -> Result<Parsed> {
        use lexopt::prelude::*;

        fn path_value(parser: &mut lexopt::Parser) -> Result<PathBuf> {
            let v = parser.value()?.string()?;
            if v.starts_with('-') {
                anyhow::bail!("Expected option value but got option name {v}");
            }
            Ok(v.into())
        }

        let mut opts = Options::default();

        let cwd = OnceCell::new();
        let mut parser = lexopt::Parser::from_iter(args);
        while let Some(arg) = parser.next()? {
            match arg {
                Short('h') | Long("help") => return Ok(Parsed::Help(Self::USAGE)),
                Short('t') | Long("theme") => opts.theme = Some(parser.value()?.parse()?),
                Long("no-watch") => opts.watch = false,
                Long("generate-config-file") => opts.gen_config_file = true,
                Long("config-dir") => opts.config_dir = Some(path_value(&mut parser)?),
                Long("data-dir") => opts.data_dir = Some(path_value(&mut parser)?),
                Long("debug") => opts.debug = true,
                Value(path) => {
                    let path = PathBuf::from(path);
                    let exists = path.exists();

                    // `path.canonicalize()` returns an error when the path does not exist. Instead, create the absolute path
                    // using current directory as a parent
                    let path = if exists {
                        path.canonicalize()?
                    } else {
                        cwd.get_or_try_init(|| env::current_dir()?.canonicalize())?.join(path)
                    };

                    if opts.init_file.is_some() || !exists || path.is_dir() {
                        opts.watch_paths.push(path);
                    } else {
                        opts.init_file = Some(path);
                    }
                }
                _ => return Err(arg.unexpected().into()),
            }
        }

        Ok(Parsed::Options(opts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmdline(args: &[&str]) -> impl Iterator<Item = String> {
        let mut c = vec!["shiba".to_string()];
        c.extend(args.iter().map(ToString::to_string));
        c.into_iter()
    }

    #[test]
    fn parse_args_ok() {
        let cur = env::current_dir().unwrap().canonicalize().unwrap();

        #[rustfmt::skip]
        let tests = [
            (
                &[][..],
                Options::default(),
            ),
            (
                &["README.md"][..],
                Options {
                    init_file: Some(cur.join("README.md")),
                    ..Default::default()
                },
            ),
            (
                &["README.md", "src"][..],
                Options {
                    init_file: Some(cur.join("README.md")),
                    watch_paths: vec![cur.join("src")],
                    ..Default::default()
                },
            ),
            (
                &["file-not-existing.md"][..],
                Options {
                    init_file: None,
                    watch_paths: vec![cur.join("file-not-existing.md")],
                    ..Default::default()
                },
            ),
            (
                &["--no-watch"][..],
                Options { watch: false, ..Default::default() },
            ),
            (
                &["--debug"][..],
                Options { debug: true, ..Default::default() },
            ),
            (
                &["--generate-config-file"][..],
                Options { gen_config_file: true, ..Default::default() },
            ),
            (
                &["--theme", "dark"][..],
                Options { theme: Some(ThemeOption::Dark), ..Default::default() },
            ),
            (
                &["--theme", "light"][..],
                Options { theme: Some(ThemeOption::Light), ..Default::default() },
            ),
            (
                &["--theme", "system"][..],
                Options { theme: Some(ThemeOption::System), ..Default::default() },
            ),
            (
                &["--config-dir", "some-dir"][..],
                Options {
                    config_dir: Some(PathBuf::from("some-dir")),
                    ..Default::default()
                },
            ),
            (
                &["--data-dir", "some-dir"][..],
                Options {
                    data_dir: Some(PathBuf::from("some-dir")),
                    ..Default::default()
                },
            ),
        ];

        for (args, want) in tests {
            match Options::parse(cmdline(args)).unwrap() {
                Parsed::Options(opts) => assert_eq!(opts, want, "args={args:?}"),
                Parsed::Help(_) => panic!("--help is returned for {args:?}"),
            }
        }
    }

    #[test]
    fn help_option() {
        match Options::parse(cmdline(&["--help"])).unwrap() {
            Parsed::Options(opts) => panic!("--help is not recognized: {opts:?}"),
            Parsed::Help(help) => assert_eq!(help, Options::USAGE),
        }
    }

    #[test]
    fn parse_unknown_option() {
        for arg in ["--foo", "-f"] {
            let err = Options::parse(cmdline(&[arg])).unwrap_err();
            let have = format!("{err}");
            let want = format!("invalid option '{arg}'");
            assert_eq!(have, want, "unexpected message {have:?}");
        }
    }

    #[test]
    fn parse_missing_option_arg() {
        for arg in ["--config-dir", "--data-dir", "--theme"] {
            let err = Options::parse(cmdline(&["--debug", arg])).unwrap_err();
            assert_eq!(
                format!("{err}"),
                format!("missing argument for option '{arg}'"),
                "unexpected message {err:?} for {arg:?}",
            );
        }
    }

    #[test]
    fn parse_invalid_option_arg() {
        for arg in ["--config-dir", "--data-dir"] {
            let err = Options::parse(cmdline(&[arg, "--debug"])).unwrap_err();
            assert_eq!(
                format!("{err}"),
                "Expected option value but got option name --debug",
                "unexpected message {err:?} for {arg:?}",
            );
        }
    }

    #[test]
    fn parse_theme_name() {
        let err = Options::parse(cmdline(&["--theme", "foo"])).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains(r#"Value for --theme must be one of "dark", "light" or "system""#),
            "unexpected message {msg:?}",
        );
    }
}
