use crate::renderer::ZoomLevel;
use anyhow::{Context, Result};
use indexmap::IndexSet;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub trait PersistentData {
    const FILE: &'static str;
}

#[derive(Debug)]
pub struct DataDir {
    path: Option<PathBuf>,
}

impl Default for DataDir {
    fn default() -> Self {
        fn data_dir() -> Option<PathBuf> {
            let mut dir = dirs::data_dir()?;
            dir.push("Shiba");
            fs::create_dir_all(&dir).ok()?;
            log::debug!("Data directory: {dir:?}");
            Some(dir)
        }
        Self { path: data_dir() }
    }
}

impl DataDir {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        Self { path: dir.is_dir().then_some(dir) }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn load<D: PersistentData + DeserializeOwned>(&self) -> Option<D> {
        let path = self.path.as_deref()?.join(D::FILE);
        let bytes = match fs::read(&path) {
            Ok(data) => data,
            Err(err) => {
                log::debug!("Could not load persistent data from {path:?}: {err}");
                return None;
            }
        };
        // serde_json::from_reader may be efficient when writing large data
        match serde_json::from_slice(&bytes) {
            Ok(data) => Some(data),
            Err(err) => {
                log::error!(
                    "Persistent data is broken. Remove {path:?} to solve this error: {err}"
                );
                None
            }
        }
    }

    pub fn save<D: PersistentData + Serialize>(&self, data: &D) -> Result<()> {
        let Some(dir) = &self.path else {
            return Ok(());
        };
        let path = dir.join(D::FILE);
        // serde_json::to_writer may be efficient when writing large data
        let s = serde_json::to_string(data)
            .with_context(|| format!("Could not serialize persistent data to {path:?}"))?;
        log::debug!("Saved persistent data at {path:?}");
        fs::write(&path, s)
            .with_context(|| format!("Could not save persistent data to file {path:?}"))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowState {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub fullscreen: bool,
    pub zoom_level: ZoomLevel,
    pub always_on_top: bool,
    pub maximized: bool,
}

impl PersistentData for WindowState {
    const FILE: &'static str = "window.json";
}

#[derive(Serialize, Debug)]
pub struct HistoryData<'a> {
    pub paths: &'a IndexSet<PathBuf>,
}

impl PersistentData for HistoryData<'_> {
    const FILE: &'static str = "history.json";
}

#[derive(Deserialize, Debug)]
pub struct HistoryDataOwned {
    pub paths: IndexSet<PathBuf>,
}

impl PersistentData for HistoryDataOwned {
    const FILE: &'static str = HistoryData::FILE;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_dir() {
        let dir = DataDir::new(Path::new("."));
        assert!(dir.path.is_some());
        let dir = DataDir::new(Path::new("this-directory-does-not-exist"));
        assert!(dir.path.is_none());
    }
}
