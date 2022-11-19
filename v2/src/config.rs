use crate::cli::Options;
use crate::renderer::KeyAction;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::time::Duration;

fn default_keymaps() -> HashMap<String, KeyAction> {
    use KeyAction::*;

    #[rustfmt::skip]
    const DEFAULT_MAPPINGS: &[(&str, KeyAction)] = &[
        ("j",         ScrollDown),
        ("k",         ScrollUp),
        ("h",         ScrollLeft),
        ("l",         ScrollRight),
        ("ctrl+b",    Back),
        ("ctrl+f",    Forward),
        ("r",         Reload),
        ("g g",       ScrollTop),
        ("G",         ScrollBottom),
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
        ("ctrl+j",    NextSection),
        ("ctrl+k",    PrevSection),
    ];

    let mut m = HashMap::new();
    for (bind, action) in DEFAULT_MAPPINGS {
        m.insert(bind.to_string(), *action);
    }
    m
}

const fn default_throttle() -> u32 {
    50
}

#[repr(transparent)]
#[derive(Deserialize, Debug, Clone)]
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
#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
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

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct Search {
    matcher: SearchMatcher,
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
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

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct Window {
    pub restore: bool,
    pub theme: WindowTheme,
}

#[non_exhaustive]
#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    file_extensions: FileExtensions,
    #[serde(default = "default_throttle")]
    debounce_throttle: u32,
    #[serde(default = "default_keymaps")]
    keymaps: HashMap<String, KeyAction>,
    #[serde(default)]
    search: Search,
    #[serde(default)]
    window: Window,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            file_extensions: FileExtensions::default(),
            debounce_throttle: default_throttle(),
            keymaps: default_keymaps(),
            search: Search::default(),
            window: Window::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        if let Some(mut config_path) = dirs::config_dir() {
            config_path.push("Shiba");
            if config_path.is_dir() {
                for file in ["config.yml", "config.yaml"] {
                    config_path.push(file);
                    match File::open(&config_path) {
                        Ok(file) => {
                            return serde_yaml::from_reader(file).with_context(|| {
                                format!("Could not parse config file as YAML: {:?}", config_path)
                            })
                        }
                        Err(err) => {
                            log::debug!(
                                "Could not read config file from {:?}: {}",
                                config_path,
                                err
                            )
                        }
                    }
                    config_path.pop();
                }
            }
        }

        log::debug!("Fallback to the default config since no config file could be loaded");
        Ok(Self::default())
    }

    pub fn merge_options(mut self, options: &Options) -> Self {
        if let Some(theme) = options.theme {
            self.window.theme = theme;
        }
        self
    }

    pub fn debounce_throttle(&self) -> Duration {
        Duration::from_millis(self.debounce_throttle as u64)
    }

    pub fn file_extensions(&self) -> &FileExtensions {
        &self.file_extensions
    }

    pub fn keymaps(&self) -> &HashMap<String, KeyAction> {
        &self.keymaps
    }

    pub fn search(&self) -> &Search {
        &self.search
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
