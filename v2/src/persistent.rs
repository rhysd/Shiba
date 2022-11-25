use crate::config::Config;
use crate::renderer::ZoomLevel;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn deserialize<'de, D: Deserialize<'de>>(data: &'de [u8], path: &Path) -> Option<D> {
    match serde_json::from_slice(data) {
        Ok(state) => Some(state),
        Err(err) => {
            log::error!(
                "Persistent data file is broken. Remove {:?} to solve this error: {}",
                path,
                err,
            );
            None
        }
    }
}

fn read_file(path: &Path) -> Option<Vec<u8>> {
    match fs::read(path) {
        Ok(data) => Some(data),
        Err(err) => {
            log::debug!("Could not load persistent data from {:?}: {}", path, err);
            None
        }
    }
}

pub trait PersistentDataSave: Serialize {
    const FILE_NAME: &'static str;
}
pub trait PersistentDataLoad: Sized {
    fn load(data_dir: &Path, config: &Config) -> Option<Self>;
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
        log::debug!("Data directory: {:?}", path);
        Self { path }
    }

    pub fn load<L: PersistentDataLoad>(&self, config: &Config) -> Option<L> {
        self.path.as_deref().and_then(|dir| L::load(dir, config))
    }

    pub fn save<S: PersistentDataSave>(&self, data: &S) -> Result<()> {
        let Some(dir) = &self.path else {
            return Ok(());
        };
        let path = dir.join(S::FILE_NAME);
        let data = serde_json::to_string(data)
            .with_context(|| format!("Could not serialize persistent data to {path:?}"))?;
        log::debug!("Saved persistent data at {:?}", path);
        fs::write(&path, &data)
            .with_context(|| format!("Could not save persistent data to file {:?}", path))
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

impl PersistentDataSave for WindowState {
    const FILE_NAME: &'static str = "window.json";
}

impl PersistentDataLoad for WindowState {
    fn load(data_dir: &Path, config: &Config) -> Option<Self> {
        if !config.window().restore {
            return None;
        }

        let path = data_dir.join(Self::FILE_NAME);
        let bytes = read_file(&path)?;
        let state = deserialize(&bytes, &path)?;

        log::debug!("Loaded window state from {:?}: {:?}", path, state);
        Some(state)
    }
}

impl<'a> SaveRecentFiles<'a> {
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

impl<'a> PersistentDataSave for SaveRecentFiles<'a> {
    const FILE_NAME: &'static str = "recent_files.json";
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

        let path = data_dir.join(SaveRecentFiles::FILE_NAME);
        let bytes = read_file(&path)?;
        let mut state = deserialize::<Self>(&bytes, &path)?;
        state.paths.retain(|p| p.exists());

        log::debug!("Loaded recent files from {:?} ({} paths)", path, state.paths.len());
        Some(state)
    }
}

#[derive(Default, Serialize, Debug)]
pub struct SaveRecentFiles<'a> {
    pub paths: Vec<&'a Path>,
}
