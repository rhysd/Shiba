use crate::cli::Options;
use crate::persistent::DataDir;
use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};
use std::time::Duration;

const DEFAULT_CONFIG_FILE: &str = include_str!("default_config.yml");

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

#[repr(transparent)]
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FileExtensions(Vec<String>);

impl Default for FileExtensions {
    fn default() -> Self {
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
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchMatcher {
    SmartCase,
    CaseSensitive,
    CaseInsensitive,
    CaseSensitiveRegex,
}

impl Default for SearchMatcher {
    fn default() -> Self {
        Self::SmartCase
    }
}

#[non_exhaustive]
#[derive(Default, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct Search {
    matcher: SearchMatcher,
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum WindowTheme {
    System,
    Dark,
    Light,
}

impl Default for WindowTheme {
    fn default() -> Self {
        Self::System
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[non_exhaustive]
#[derive(Default, Deserialize, Debug, PartialEq, Eq)]
pub struct Window {
    pub restore: bool,
    pub theme: WindowTheme,
    pub always_on_top: bool,
    pub default_size: Option<WindowSize>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
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
pub struct Preview {
    highlight: PreviewHighlight,
    css: Option<PathBuf>,
    recent_files: usize,
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
    if !s.starts_with(PREFIX) {
        return Ok(Some(PathBuf::from(s)));
    }

    let Some(mut path) = dirs::home_dir() else {
        return Ok(None);
    };

    path.push(&s[2..]);
    if !path.is_dir() {
        log::error!("Path {:?} in config is not a directory", path);
        return Ok(None);
    }

    Ok(Some(path))
}

#[derive(Default, Deserialize, Debug, PartialEq, Eq)]
pub struct Dialog {
    #[serde(deserialize_with = "resolve_path")]
    default_dir: Option<PathBuf>,
}

impl Dialog {
    pub fn default_dir(&self) -> Option<&Path> {
        self.default_dir.as_deref()
    }
}

#[non_exhaustive]
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct ConfigData {
    watch: Watch,
    keymaps: HashMap<String, KeyAction>,
    search: Search,
    window: Window,
    preview: Preview,
    dialog: Dialog,
}

impl Default for ConfigData {
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

impl ConfigData {
    fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let mut path = path.into();

        if path.is_dir() {
            for file in ["config.yml", "config.yaml"] {
                path.push(file);

                match fs::read(&path) {
                    Ok(bytes) =>
                        return serde_yaml::from_slice(&bytes)
                            .with_context(|| format!("Could not parse config file as YAML: {:?}. To reset config file, try --generate-config-file", path)),
                    Err(err) => {
                        log::debug!("Could not read config file from {:?}: {}", path, err);
                    }
                }

                path.pop();
            }
        }

        log::debug!("config.yml nor config.yaml was found in {path:?}. Using the default config");
        Ok(Self::default())
    }

    fn generate_default_config(config_dir: &Path) -> Result<()> {
        fs::create_dir_all(config_dir).with_context(|| {
            format!("Could not create directory for generating config file at {:?}", config_dir)
        })?;

        let config_path = config_dir.join("config.yml");
        fs::write(&config_path, DEFAULT_CONFIG_FILE)
            .with_context(|| format!("Could not generate config file at {:?}", &config_path))?;

        log::info!("Generated the default config file at {:?}", config_path);
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct Config {
    data: ConfigData,
    path: Option<PathBuf>,
    data_dir: DataDir,
    debug: bool,
    init_file: Option<PathBuf>,
}

impl Config {
    pub fn load(mut options: Options) -> Result<Self> {
        let config_dir = mem::take(&mut options.config_dir).or_else(|| {
            let mut dir = dirs::config_dir()?;
            dir.push("Shiba");
            Some(dir)
        });

        let mut data = if options.gen_config_file {
            if let Some(dir) = &config_dir {
                ConfigData::generate_default_config(dir)?;
            } else {
                anyhow::bail!("Config directory cannot be determined on this system. Config file is not available");
            }
            ConfigData::default()
        } else if let Some(dir) = &config_dir {
            ConfigData::load(dir)?
        } else {
            log::debug!("Config directory does not exist. Using the default config");
            ConfigData::default()
        };

        if let Some(theme) = options.theme {
            data.window.theme = theme;
        }

        let data_dir = if let Some(dir) = &options.data_dir {
            DataDir::custom_dir(dir)
        } else {
            DataDir::new()
        };

        Ok(Self {
            data,
            path: config_dir,
            data_dir,
            debug: options.debug,
            init_file: options.init_file,
        })
    }

    pub fn config_file(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn data_dir(&self) -> &DataDir {
        &self.data_dir
    }

    pub fn watch(&self) -> &Watch {
        &self.data.watch
    }

    pub fn keymaps(&self) -> &HashMap<String, KeyAction> {
        &self.data.keymaps
    }

    pub fn search(&self) -> &Search {
        &self.data.search
    }

    pub fn window(&self) -> &Window {
        &self.data.window
    }

    pub fn preview(&self) -> &Preview {
        &self.data.preview
    }

    pub fn dialog(&self) -> &Dialog {
        &self.data.dialog
    }

    pub fn max_recent_files(&self) -> usize {
        self.data.preview.recent_files
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    pub fn take_init_file(&mut self) -> Option<PathBuf> {
        mem::take(&mut self.init_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_default_config() {
        let cfg: ConfigData = serde_yaml::from_str(DEFAULT_CONFIG_FILE).unwrap();
        assert_eq!(cfg, ConfigData::default());
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
}
