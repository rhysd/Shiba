use crate::config::Config;
use crate::renderer::ZoomLevel;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub trait PersistentDataLoad: Sized {
    fn load(data_dir: &Path, config: &Config) -> Option<Self>;
}
pub trait PersistentDataWrite: Sized {
    fn write(&self, data_dir: &Path) -> Result<()>;
}

pub struct PersistentData {
    data_dir: Option<PathBuf>,
}

impl PersistentData {
    pub fn new() -> Self {
        fn data_dir() -> Option<PathBuf> {
            let mut dir = dirs::data_dir()?;
            dir.push("Shiba");
            fs::create_dir_all(&dir).ok()?;
            Some(dir)
        }

        let data_dir = data_dir();
        if data_dir.is_none() {
            log::debug!(
                "Data directory could not be prepared. Persistent data will not be available"
            );
        }

        Self { data_dir }
    }

    pub fn load<L: PersistentDataLoad>(&self, config: &Config) -> Option<L> {
        if let Some(dir) = &self.data_dir {
            L::load(dir, config)
        } else {
            None
        }
    }

    pub fn write<W: PersistentDataWrite>(&self, data: &W) -> Result<()> {
        if let Some(dir) = &self.data_dir {
            data.write(dir)
        } else {
            Ok(())
        }
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

impl PersistentDataLoad for WindowState {
    fn load(data_dir: &Path, config: &Config) -> Option<Self> {
        if !config.window().restore {
            return None;
        }

        let path = data_dir.join("window.json");
        log::debug!("Loading window state from {:?}", path);

        let data = match fs::read(&path) {
            Ok(data) => data,
            Err(err) => {
                log::debug!("Could not load window state from {:?}: {}", path, err);
                return None;
            }
        };

        match serde_json::from_slice(&data) {
            Ok(state) => Some(state),
            Err(err) => {
                log::error!(
                    "Window state file is broken. Remove {:?} to solve this error: {}",
                    path,
                    err,
                );
                None
            }
        }
    }
}

impl PersistentDataWrite for WindowState {
    fn write(&self, data_dir: &Path) -> Result<()> {
        let path = data_dir.join("window.json");
        let data = serde_json::to_string(&self)
            .with_context(|| format!("Could not serialize window state file {path:?}"))?;
        log::debug!("Saving window state to {:?}", path);
        Ok(fs::write(&path, data)?)
    }
}

#[derive(Default, Deserialize, Debug)]
pub struct LoadRecentFiles {
    pub paths: Vec<PathBuf>,
}

impl PersistentDataLoad for LoadRecentFiles {
    fn load(data_dir: &Path, config: &Config) -> Option<Self> {
        let max_size = config.preview().recent_files();
        if max_size == 0 {
            return None;
        }

        let path = data_dir.join("recent_files.json");
        log::debug!("Loading recent files from {:?}", path);

        let data = match fs::read(&path) {
            Ok(data) => data,
            Err(err) => {
                log::debug!("Could not load recent files from {path:?}: {err}");
                return None;
            }
        };

        match serde_json::from_slice::<Self>(&data) {
            Ok(mut state) => {
                if state.paths.len() > max_size {
                    state.paths.drain(0..state.paths.len() - max_size);
                }
                Some(state)
            }
            Err(err) => {
                log::error!(
                    "Window state file is broken. Remove {path:?} to solve this error: {err}",
                );
                None
            }
        }
    }
}

#[derive(Default, Serialize, Debug)]
pub struct WriteRecentFiles<'a> {
    pub paths: Vec<&'a Path>,
}

impl<'a> WriteRecentFiles<'a> {
    pub fn new(iter: impl Iterator<Item = &'a Path>, max_size: usize) -> Self {
        let mut seen = HashSet::new();
        let mut paths = vec![];
        for path in iter {
            if paths.len() >= max_size {
                return Self { paths };
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

impl<'a> PersistentDataWrite for WriteRecentFiles<'a> {
    fn write(&self, data_dir: &Path) -> Result<()> {
        let path = data_dir.join("recent_files.json");
        let data = serde_json::to_string(&self)
            .with_context(|| format!("Could not serialize recent files to {path:?}"))?;
        log::debug!("Saving recent files to {:?}", path);
        Ok(fs::write(&path, data)?)
    }
}
