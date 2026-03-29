use anyhow::{Error, Result};
use once_cell::unsync::OnceCell; // For OnceCell::get_or_try_init
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
    Version(&'static str),
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct Options {
    pub debug: bool,
    pub init_files: Vec<PathBuf>,
    pub watch_paths: Vec<PathBuf>,
    pub watch: bool,
    pub restore: bool,
    pub theme: Option<ThemeOption>,
    pub gen_config_file: bool,
    pub config_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
    pub process_singleton: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            debug: false,
            init_files: vec![],
            watch_paths: vec![],
            watch: true,
            restore: true,
            theme: None,
            gen_config_file: false,
            config_dir: None,
            data_dir: None,
            process_singleton: true,
        }
    }
}

impl Options {
    const USAGE: &'static str = r#"Usage: shiba [OPTIONS...] [PATH...]

Shiba is a markdown browser to preview documents with your favorite text editor, designed for
simplicity, performance, and keyboard-friendly navigations.

Options:

    -o, --open FILE             Open the file with a new window. This option is repeatable
    -t, --theme THEME           Window theme ("system" (default), "dark" or "light")
        --no-watch              Disable to watch file changes
        --no-restore            Do not restore the previous window state
        --generate-config-file  Generate the default config file overwriting an existing file
        --config-dir PATH       Change the config directory path
        --data-dir PATH         Change the application data directory path
        --no-proc-singleton     Don't reuse an existing application process
        --debug                 Enable debug features
    -h, --help                  Print this help
        --version               Print application version

Arguments:

    PATH...                     Paths to the files and directories to watch. The first file is
                                opened in a preview window. Rest of paths are shown in the preview
                                window when they are modified (e.g. edited by a text editor) next
                                time. If you want to open them in multiple windows, use --open or -o
                                option.

Examples:

    $ shiba file.md
        Opens `file.md` file in a preview window and tracks the file changes.

    $ shiba dir/
        Tracks all files in the `dir` directory. When one of them is modified, it's opened in a
        preview window.

    $ shiba file1.md file2.md dir1 dir2
        Tracks `file1.md`, `file2.md`, files in `dir1` directory, and files in `dir2` directory.
        `file1.md` file is opened in a preview window.

    $ shiba file1.md -o file2.md -o file3.md
        Opens the three files in three windows respectively and tracks the file changes. The first
        file path implies --open option so you don't need to specify it.

    $ shiba
        Opens an empty window. You can open files from key shortcuts, menu items, file picker, etc.

Document:

    https://github.com/rhysd/Shiba/README.md
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

        let mut opts = Self::default();

        let cwd = OnceCell::new();
        let mut parser = lexopt::Parser::from_iter(args);
        while let Some(arg) = parser.next()? {
            match arg {
                Short('h') | Long("help") => return Ok(Parsed::Help(Self::USAGE)),
                Long("version") => return Ok(Parsed::Version(env!("CARGO_PKG_VERSION"))),
                Short('t') | Long("theme") => opts.theme = Some(parser.value()?.parse()?),
                Long("no-watch") => opts.watch = false,
                Long("no-restore") => opts.restore = false,
                Long("generate-config-file") => opts.gen_config_file = true,
                Long("config-dir") => opts.config_dir = Some(path_value(&mut parser)?),
                Long("data-dir") => opts.data_dir = Some(path_value(&mut parser)?),
                Long("no-proc-singleton") => opts.process_singleton = false,
                Long("debug") => opts.debug = true,
                Short('o') | Long("open") => {
                    let path = path_value(&mut parser)?;
                    let path = match path.metadata() {
                        Ok(md) if md.is_dir() => anyhow::bail!(
                            "--open only works with files but directory found: {path:?}",
                        ),
                        Ok(_) => path.canonicalize()?,
                        Err(err) => {
                            let err = Error::new(err)
                                .context(format!("Could not open the file for --open: {path:?}"));
                            return Err(err);
                        }
                    };
                    opts.init_files.push(path)
                }
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

                    if !opts.init_files.is_empty() || !exists || path.is_dir() {
                        opts.watch_paths.push(path);
                    } else {
                        opts.init_files.push(path);
                    }
                }
                _ => return Err(arg.unexpected().into()),
            }
        }

        log::debug!("Parsed command line options: {opts:?}");
        Ok(Parsed::Options(opts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmdline(args: &[&str]) -> impl Iterator<Item = String> + use<> {
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
                    init_files: vec![cur.join("README.md")],
                    ..Default::default()
                },
            ),
            (
                &["README.md", "src"][..],
                Options {
                    init_files: vec![cur.join("README.md")],
                    watch_paths: vec![cur.join("src")],
                    ..Default::default()
                },
            ),
            (
                &["file-not-existing.md"][..],
                Options {
                    watch_paths: vec![cur.join("file-not-existing.md")],
                    ..Default::default()
                },
            ),
            (
                &["file-not-existing.md", "README.md"][..],
                Options {
                    init_files: vec![cur.join("README.md")],
                    watch_paths: vec![cur.join("file-not-existing.md")],
                    ..Default::default()
                },
            ),
            (
                &["--no-watch"][..],
                Options { watch: false, ..Default::default() },
            ),
            (
                &["--no-restore"][..],
                Options { restore: false, ..Default::default() },
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
                    config_dir: Some("some-dir".into()),
                    ..Default::default()
                },
            ),
            (
                &["--data-dir", "some-dir"][..],
                Options {
                    data_dir: Some("some-dir".into()),
                    ..Default::default()
                },
            ),
            (
                &["README.md", "-o", "CHANGELOG.md", "--open", "LICENSE"][..],
                Options {
                    init_files: vec![
                        cur.join("README.md"),
                        cur.join("CHANGELOG.md"),
                        cur.join("LICENSE"),
                    ],
                    ..Default::default()
                },
            ),
            (
                &["-o", "README.md", "CHANGELOG.md", "-o", "LICENSE"][..],
                Options {
                    init_files: vec![
                        cur.join("README.md"),
                        cur.join("LICENSE"),
                    ],
                    watch_paths: vec![cur.join("CHANGELOG.md")],
                    ..Default::default()
                },
            ),
            (
                &["--no-proc-singleton"][..],
                Options { process_singleton: false, ..Default::default() },
            ),
        ];

        for (args, want) in tests {
            match Options::parse(cmdline(args)).unwrap() {
                Parsed::Options(opts) => assert_eq!(opts, want, "args={args:?}"),
                p => panic!("unexpected parse result: {p:?}"),
            }
        }
    }

    #[test]
    fn help_option() {
        match Options::parse(cmdline(&["--help"])).unwrap() {
            Parsed::Help(help) => assert_eq!(help, Options::USAGE),
            p => panic!("unexpected parse result: {p:?}"),
        }
    }

    #[test]
    fn version_option() {
        match Options::parse(cmdline(&["--version"])).unwrap() {
            Parsed::Version(v) => assert_eq!(v, env!("CARGO_PKG_VERSION")),
            p => panic!("unexpected parse result: {p:?}"),
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
        for arg in ["--config-dir", "--data-dir", "--theme", "--open", "-o"] {
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
        for arg in ["--config-dir", "--data-dir", "--open", "-o"] {
            let err = Options::parse(cmdline(&[arg, "--debug"])).unwrap_err();
            assert_eq!(
                format!("{err}"),
                "Expected option value but got option name --debug",
                "unexpected message {err:?} for {arg:?}",
            );
        }
    }

    #[test]
    fn parse_invalid_theme_name() {
        let err = Options::parse(cmdline(&["--theme", "foo"])).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains(r#"Value for --theme must be one of "dark", "light" or "system""#),
            "unexpected message {msg:?}",
        );
    }

    #[test]
    fn parse_invalid_open_arg() {
        for (arg, expected) in [
            (".", "--open only works with files but directory found"),
            ("not-existing.md", "Could not open the file for --open"),
            ("", "Could not open the file for --open"),
        ] {
            let err = Options::parse(cmdline(&["--open", arg])).unwrap_err();
            let msg = format!("{err}");
            assert!(
                msg.contains(expected),
                "argument {arg:?} does not cause expected message {expected:?}: {msg:?}",
            );
        }
    }
}
