use crate::cli::{Options, ThemeOption};
use crate::persistent::DataDir;
use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyAction {
    Forward,
    Back,
    Reload,
    OpenFile,
    OpenDir,
    ScrollDown,
    ScrollUp,
    ScrollLeft,
    ScrollRight,
    ScrollPageDown,
    ScrollPageUp,
    ScrollTop,
    ScrollBottom,
    Search,
    NextSearch,
    PrevSearch,
    ScrollNextSection,
    ScrollPrevSection,
    Outline,
    History,
    Help,
    ZoomIn,
    ZoomOut,
    ShowMenu,
    ToggleMenuBar,
    Quit,
}

#[rustfmt::skip]
const DEFAULT_KEY_MAPPINGS: &[(&str, KeyAction)] = {
    use KeyAction::*;
    &[
        ("j",         ScrollDown),
        ("k",         ScrollUp),
        ("h",         ScrollLeft),
        ("l",         ScrollRight),
        ("ctrl+b",    Back),
        ("ctrl+f",    Forward),
        ("ctrl+o",    OpenFile),
        ("ctrl+d",    ScrollPageDown),
        ("ctrl+u",    ScrollPageUp),
        ("down",      ScrollDown),
        ("up",        ScrollUp),
        ("left",      ScrollLeft),
        ("right",     ScrollRight),
        ("pagedown",  ScrollPageDown),
        ("pageup",    ScrollPageUp),
        ("ctrl+down", ScrollBottom),
        ("ctrl+up",   ScrollTop),
        ("ctrl+j",    ScrollNextSection),
        ("ctrl+k",    ScrollPrevSection),
        ("?",         Help),
    ]
};

const DEFAULT_CONFIG_FILE_NAME: &str = "config.yml";
const CONFIG_FILE_NAMES: [&str; 2] = [DEFAULT_CONFIG_FILE_NAME, "config.yaml"];

#[repr(transparent)]
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FileExtensions(Vec<String>);

impl Default for FileExtensions {
    fn default() -> Self {
        // Note: Adding more extentions may be useful.
        // - Other extentions historically used (.mdwn, .mdown, .mkdn, .mkdown)
        // - Some tools have their own extentions built on top of Markdown (.livemd, .ronn, .scd)
        //
        // See: https://github.com/github-linguist/linguist/blob/e51c227048a02a8a1b0fae6e72214e7c5f327c73/lib/linguist/languages.yml#L4564-L4575
        Self(vec!["md".into(), "mkd".into(), "markdown".into()])
    }
}

impl FileExtensions {
    pub fn matches(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            self.0.iter().any(|e| ext == e.as_str())
        } else {
            false
        }
    }

    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}

#[non_exhaustive]
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Watch {
    file_extensions: FileExtensions,
    debounce_throttle: u32,
}

impl Default for Watch {
    fn default() -> Self {
        Self { file_extensions: Default::default(), debounce_throttle: 50 }
    }
}

impl Watch {
    pub fn debounce_throttle(&self) -> Duration {
        Duration::from_millis(self.debounce_throttle as u64)
    }

    pub fn file_extensions(&self) -> &FileExtensions {
        &self.file_extensions
    }
}

#[non_exhaustive]
#[derive(Deserialize, Serialize, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchMatcher {
    #[default]
    SmartCase,
    CaseSensitive,
    CaseInsensitive,
    CaseSensitiveRegex,
}

#[non_exhaustive]
#[derive(Deserialize, Serialize, Default, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Search {
    matcher: SearchMatcher,
}

#[derive(Deserialize, Serialize, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowTheme {
    #[default]
    System,
    Dark,
    Light,
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[non_exhaustive]
#[derive(Default, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Window {
    pub restore: bool,
    pub theme: WindowTheme,
    pub always_on_top: bool,
    pub default_size: Option<WindowSize>,
    pub menu_bar: bool,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PreviewHighlight {
    pub dark: String,
    pub light: String,
}

impl Default for PreviewHighlight {
    fn default() -> Self {
        Self { dark: "GitHub Dark".to_string(), light: "GitHub".to_string() }
    }
}

#[non_exhaustive]
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    highlight: PreviewHighlight,
    css: Option<PathBuf>,
    pub recent_files: usize,
}

impl Default for Preview {
    fn default() -> Self {
        Self { highlight: PreviewHighlight::default(), css: None, recent_files: 100 }
    }
}

impl Preview {
    pub fn highlight(&self) -> &PreviewHighlight {
        &self.highlight
    }

    pub fn css_path(&self) -> Option<&Path> {
        self.css.as_deref()
    }
}

fn resolve_path<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<PathBuf>, D::Error> {
    #[cfg(not(target_os = "windows"))]
    const PREFIX: &str = "~/";
    #[cfg(target_os = "windows")]
    const PREFIX: &str = "~\\";

    let s = String::deserialize(deserializer)?;
    if &s == "null" {
        return Ok(None);
    }

    let path = if let Some(rel) = s.strip_prefix(PREFIX) {
        let Some(mut home) = dirs::home_dir() else {
            return Ok(None);
        };
        home.push(rel);
        home
    } else {
        PathBuf::from(s)
    };

    if !path.is_dir() {
        log::error!("Path {:?} in config is not a directory", path);
        return Ok(None);
    }

    Ok(Some(path))
}

#[derive(Default, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Dialog {
    #[serde(deserialize_with = "resolve_path")]
    default_dir: Option<PathBuf>,
}

impl Dialog {
    pub fn default_dir(&self) -> Result<Cow<'_, Path>> {
        if let Some(path) = self.default_dir.as_deref() {
            return Ok(path.into());
        }
        let dir = env::current_dir().context("Error while opening a file dialog")?;

        // When this app is started via Shiba.app, the current directory is `/` but it is not convenient as an initial
        // directory for open dialog.
        #[cfg(target_os = "macos")]
        if dir.parent().is_none() {
            if let Some(dir) = dirs::document_dir() {
                return Ok(dir.into());
            }
        }

        Ok(dir.into())
    }
}

#[non_exhaustive]
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UserConfig {
    watch: Watch,
    keymaps: HashMap<String, KeyAction>,
    search: Search,
    window: Window,
    preview: Preview,
    dialog: Dialog,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            watch: Watch::default(),
            keymaps: DEFAULT_KEY_MAPPINGS.iter().map(|(b, a)| (b.to_string(), *a)).collect(),
            search: Search::default(),
            window: Window::default(),
            preview: Preview::default(),
            dialog: Dialog::default(),
        }
    }
}

impl UserConfig {
    const DEFAULT_CONFIG_YAML: &'static str = include_str!("assets/default_config.yml");

    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut path = fs::read_link(path).unwrap_or_else(|_| path.to_path_buf());

        if path.is_dir() {
            for file_name in CONFIG_FILE_NAMES {
                path.push(file_name);

                let file = if let Ok(path) = fs::read_link(&path) {
                    Cow::Owned(path)
                } else {
                    Cow::Borrowed(&path)
                };
                let file = file.as_ref();

                match fs::read(file) {
                    Ok(bytes) => {
                        let config = serde_yaml::from_slice(&bytes)
                            .with_context(|| {
                                format!("Could not parse a configuration file at {file:?}. To reset config file, try --generate-config-file")
                            })?;
                        log::debug!("Loaded the user configuration from {file:?}");
                        return Ok(config);
                    }
                    Err(err) => log::debug!("Could not read config file from {file:?}: {err}"),
                }

                path.pop();
            }
        }

        log::debug!(
            "Neither config.yml nor config.yaml was found in {path:?}. Using the default config"
        );
        Ok(Self::default())
    }

    fn generate_default_config(config_dir: &Path) -> Result<PathBuf> {
        fs::create_dir_all(config_dir).with_context(|| {
            format!("Could not create directory for generating config file at {:?}", config_dir)
        })?;

        let config_path = config_dir.join(DEFAULT_CONFIG_FILE_NAME);
        fs::write(&config_path, Self::DEFAULT_CONFIG_YAML)
            .with_context(|| format!("Could not generate config file at {:?}", &config_path))?;

        log::info!("Generated the default config file at {:?}", config_path);
        Ok(config_path)
    }
}

#[derive(Default, Debug)]
pub struct Config {
    user_config: UserConfig,
    path: Option<PathBuf>,
    data_dir: DataDir,
    debug: bool,
}

impl Config {
    pub fn load(mut options: Options) -> Result<Self> {
        let config_dir = mem::take(&mut options.config_dir).or_else(|| {
            let mut dir = dirs::config_dir()?;
            dir.push("Shiba");
            Some(dir)
        });

        let mut user_config = if options.gen_config_file {
            if let Some(dir) = &config_dir {
                UserConfig::generate_default_config(dir)?;
            } else {
                anyhow::bail!(
                    "Config directory cannot be determined on this system. Config file is not available",
                );
            }
            UserConfig::default()
        } else if let Some(dir) = &config_dir {
            UserConfig::load(dir)?
        } else {
            log::debug!("Config directory does not exist. Using the default config");
            UserConfig::default()
        };

        if let Some(theme) = options.theme {
            // CLI option has higher priority
            user_config.window.theme = match theme {
                ThemeOption::System => WindowTheme::System,
                ThemeOption::Dark => WindowTheme::Dark,
                ThemeOption::Light => WindowTheme::Light,
            };
        }

        if !options.restore {
            user_config.window.restore = false;
        }

        let data_dir = if let Some(dir) = mem::take(&mut options.data_dir) {
            DataDir::new(dir)
        } else {
            DataDir::default()
        };

        Ok(Self { user_config, path: config_dir, data_dir, debug: options.debug })
    }

    pub fn config_dir(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn config_file(&self) -> Result<PathBuf> {
        let Some(dir) = self.config_dir() else {
            anyhow::bail!("Configuration directory cannot be determined. Try --config-dir");
        };
        if let Some(path) = CONFIG_FILE_NAMES.iter().find_map(|file| {
            let path = dir.join(file);
            path.is_file().then_some(path)
        }) {
            Ok(path)
        } else {
            UserConfig::generate_default_config(dir)
        }
    }

    pub fn data_dir(&self) -> &DataDir {
        &self.data_dir
    }

    pub fn watch(&self) -> &Watch {
        &self.user_config.watch
    }

    pub fn keymaps(&self) -> &HashMap<String, KeyAction> {
        &self.user_config.keymaps
    }

    pub fn search(&self) -> &Search {
        &self.user_config.search
    }

    pub fn window(&self) -> &Window {
        &self.user_config.window
    }

    pub fn preview(&self) -> &Preview {
        &self.user_config.preview
    }

    pub fn dialog(&self) -> &Dialog {
        &self.user_config.dialog
    }

    pub fn debug(&self) -> bool {
        self.debug
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::RwLock;

    const CONFIG_OK: &str = include_str!("testdata/config/ok/config.yml");

    fn test_config_dir(name: &str) -> PathBuf {
        #[cfg(not(target_os = "windows"))]
        const ROOT: &str = "src/testdata/config";
        #[cfg(target_os = "windows")]
        const ROOT: &str = r"src\testdata\config";
        Path::new(ROOT).join(name)
    }

    // Lock to call `env::set_var` safely which was made unsafe since Rust 2024
    static ENV_LOCK: RwLock<()> = RwLock::new(());

    struct Env<'a>((&'a str, Option<String>));

    impl<'a> Env<'a> {
        #[allow(dead_code)]
        fn new(name: &'a str, value: &str) -> Self {
            let saved = env::var(name).ok();
            // SAFETY: Writing to env vars is guarded by `ENV_LOCK`
            unsafe {
                env::set_var(name, value);
            }
            Self((name, saved))
        }
    }

    impl Drop for Env<'_> {
        fn drop(&mut self) {
            let Self((name, saved)) = &self;
            if let Some(saved) = saved {
                // SAFETY: Writing to env vars is guarded by `ENV_LOCK`
                unsafe {
                    env::set_var(name, saved);
                }
            }
        }
    }

    #[test]
    fn generated_default_config() {
        let cfg: UserConfig = serde_yaml::from_str(UserConfig::DEFAULT_CONFIG_YAML).unwrap();
        assert_eq!(cfg, UserConfig::default());
    }

    #[test]
    fn default_key_mappings() {
        let mut m = HashMap::new();
        for (bind, a1) in DEFAULT_KEY_MAPPINGS {
            if let Some(a2) = m.get(bind) {
                panic!("default mapping {} conflicts: {:?} vs {:?}", bind, *a1, *a2);
            }
            if let Some(i) = bind.find('+') {
                let modifier = &bind[..i];
                assert!(matches!(modifier, "ctrl" | "alt"), "invalid modifier {:?}", modifier);
            }
            m.insert(*bind, *a1);
        }
    }

    #[test]
    fn match_file_extensions() {
        let exts = FileExtensions::default();
        assert!(exts.matches(Path::new("foo.md")));
        assert!(exts.matches(Path::new("foo.mkd")));
        assert!(exts.matches(Path::new("foo.markdown")));
        assert!(exts.matches(Path::new("/path/to/foo.md")));
        assert!(exts.matches(Path::new("/path/to/foo.mkd")));
        assert!(exts.matches(Path::new("/path/to/foo.markdown")));
        assert!(!exts.matches(Path::new("")));
        assert!(!exts.matches(Path::new("foo")));
        assert!(!exts.matches(Path::new("foo.txt")));
        assert!(!exts.matches(Path::new("/path/to/foo")));
        assert!(!exts.matches(Path::new("/path/to/foo.txt")));
    }

    #[test]
    fn load_config_from_option_path() {
        let _lock = ENV_LOCK.read().unwrap();

        let expected: UserConfig = serde_yaml::from_str(CONFIG_OK).unwrap();
        let dir = test_config_dir("ok");
        let opts = Options {
            config_dir: Some(dir.clone()),
            data_dir: Some(dir.clone()),
            ..Default::default()
        };

        let cfg = Config::load(opts).unwrap();
        assert!(!cfg.debug());
        assert_eq!(cfg.data_dir().path(), Some(dir.as_path()));
        assert_eq!(cfg.config_dir(), Some(dir.as_path()));
        assert_eq!(expected, cfg.user_config);
    }

    // XDG directory environment variables are only referred on Linux
    #[cfg(target_os = "linux")]
    #[test]
    fn load_config_from_xdg_config_dir() {
        let _lock = ENV_LOCK.read().unwrap(); // Writing to env vars must be in a single thread from Rust 2024

        let expected: UserConfig = serde_yaml::from_str(CONFIG_OK).unwrap();
        // XDG environment variable must be absolute paths
        let mut dir = env::current_dir().unwrap();
        dir.push("src");
        dir.push("testdata");
        dir.push("config");
        dir.push("xdg");

        let cfg = {
            let _config_env = Env::new("XDG_CONFIG_HOME", &dir.to_string_lossy());
            let _data_env = Env::new("XDG_DATA_HOME", &dir.to_string_lossy());
            Config::load(Options::default()).unwrap()
        };

        dir.push("Shiba");
        assert_eq!(cfg.data_dir().path(), Some(dir.as_path()));
        assert_eq!(cfg.config_dir(), Some(dir.as_path()));
        assert_eq!(expected, cfg.user_config);
    }

    #[test]
    fn reflect_option_in_config() {
        let _lock = ENV_LOCK.read().unwrap();

        let dir = test_config_dir("ok");
        let opts = Options {
            debug: true,
            theme: Some(ThemeOption::Light), // Theme in config is overwritten
            config_dir: Some(dir.clone()),
            data_dir: Some(dir.clone()),
            ..Default::default()
        };
        let cfg = Config::load(opts).unwrap();
        assert!(cfg.debug());
        assert_eq!(cfg.window().theme, WindowTheme::Light);
    }

    #[test]
    fn unknown_field_in_config() {
        let _lock = ENV_LOCK.read().unwrap();

        let dir = test_config_dir("unknown_field");
        let opts = Options { config_dir: Some(dir), ..Default::default() };
        let err = Config::load(opts).unwrap_err();
        let msg = format!("{}", err.source().unwrap());
        assert!(msg.contains("unknown field `unknown_field`"), "message={msg:?}");
    }

    #[test]
    fn missing_field_in_config() {
        let _lock = ENV_LOCK.read().unwrap();

        let dir = test_config_dir("missing_field");
        let opts = Options { config_dir: Some(dir), ..Default::default() };
        let err = Config::load(opts).unwrap_err();
        let msg = format!("{}", err.source().unwrap());
        assert!(msg.contains("missing field"), "message={msg:?}");
    }

    #[test]
    fn no_user_config() {
        let _lock = ENV_LOCK.read().unwrap();

        let dir = test_config_dir("no_config");
        let opts = Options { config_dir: Some(dir), ..Default::default() };
        let cfg = Config::load(opts).unwrap();
        assert_eq!(cfg.user_config, UserConfig::default()); // When no config is found, load the default config
    }

    #[test]
    fn symlink_config_dir() {
        let _lock = ENV_LOCK.read().unwrap();

        let dir = test_config_dir("symlink_dir");
        let opts = Options { config_dir: Some(dir), ..Default::default() };
        let cfg = Config::load(opts).unwrap();
        let expected: UserConfig = serde_yaml::from_str(CONFIG_OK).unwrap();
        assert_eq!(cfg.user_config, expected); // When no config is found, load the default config
    }

    #[test]
    fn symlink_config_file() {
        let _lock = ENV_LOCK.read().unwrap();

        let dir = test_config_dir("symlink_config");
        let opts = Options { config_dir: Some(dir), ..Default::default() };
        let cfg = Config::load(opts).unwrap();
        let expected: UserConfig = serde_yaml::from_str(CONFIG_OK).unwrap();
        assert_eq!(cfg.user_config, expected); // When no config is found, load the default config
    }
}
