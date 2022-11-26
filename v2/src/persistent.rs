use crate::config::Config;
use crate::renderer::ZoomLevel;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub trait PersistentData: DeserializeOwned {
    const FILE_NAME: &'static str; // Path::new is not const yet
    type Serialize<'s>: Serialize;
    fn is_enabled(config: &Config) -> bool;
    fn configure(&mut self, _config: &Config) {}
}

pub struct DataDir {
    path: Option<PathBuf>,
}

impl DataDir {
    pub fn new() -> Self {
        fn data_dir() -> Option<PathBuf> {
            let mut dir = dirs::data_dir()?;
            dir.push("Shiba");
            fs::create_dir_all(&dir).ok()?;
            Some(dir)
        }

        let path = data_dir();
        log::debug!("Data directory: {path:?}");
        Self { path }
    }

    pub fn load<D: PersistentData>(&self, config: &Config) -> Option<D> {
        if !D::is_enabled(config) {
            return None;
        }

        let path = self.path.as_deref()?.join(D::FILE_NAME);
        let bytes = match fs::read(&path) {
            Ok(data) => data,
            Err(err) => {
                log::debug!("Could not load persistent data from {path:?}: {err}");
                return None;
            }
        };
        let mut data: D = match serde_json::from_slice(&bytes) {
            Ok(data) => data,
            Err(err) => {
                log::error!(
                    "Persistent data is broken. Remove {path:?} to solve this error: {err}"
                );
                return None;
            }
        };

        data.configure(config);
        log::debug!("Loaded persistent data from {path:?}");
        Some(data)
    }

    pub fn save<'s, D: PersistentData>(&self, data: &D::Serialize<'s>) -> Result<()> {
        let Some(dir) = &self.path else {
            return Ok(());
        };
        let path = dir.join(D::FILE_NAME);
        let data = serde_json::to_string(data)
            .with_context(|| format!("Could not serialize persistent data to {path:?}"))?;
        log::debug!("Saved persistent data at {path:?}");
        fs::write(&path, &data)
            .with_context(|| format!("Could not save persistent data to file {path:?}"))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub zoom_level: ZoomLevel,
}

impl PersistentData for WindowState {
    const FILE_NAME: &'static str = "window.json";

    type Serialize<'s> = WindowState;

    fn is_enabled(config: &Config) -> bool {
        config.window().restore
    }
}

#[derive(Serialize, Debug)]
pub struct SerializeRecentFiles<'a> {
    pub paths: Vec<&'a Path>,
}

impl<'a> SerializeRecentFiles<'a> {
    pub fn new(iter: impl Iterator<Item = &'a Path>, max_size: usize) -> Self {
        let mut seen = HashSet::new();
        let mut paths = vec![];
        for path in iter {
            if paths.len() >= max_size {
                break;
            }
            if seen.contains(path) {
                continue;
            }
            seen.insert(path);
            paths.push(path);
        }
        Self { paths }
    }
}

#[derive(Deserialize, Debug)]
pub struct RecentFiles {
    pub paths: Vec<PathBuf>,
}

impl PersistentData for RecentFiles {
    const FILE_NAME: &'static str = "recent_files.json";

    type Serialize<'s> = SerializeRecentFiles<'s>;

    fn is_enabled(config: &Config) -> bool {
        config.preview().recent_files() > 0
    }

    fn configure(&mut self, config: &Config) {
        self.paths.retain(|p| p.exists());
        let max = config.preview().recent_files();
        if self.paths.len() > max {
            self.paths.drain(0..self.paths.len() - max);
        }
    }
}
