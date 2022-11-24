use crate::renderer::ZoomLevel;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub zoom_level: ZoomLevel,
}

impl WindowState {
    fn path() -> Option<PathBuf> {
        let mut path = dirs::data_dir()?;
        path.push("Shiba");
        path.push("window.json");
        Some(path)
    }

    pub fn load() -> Option<Self> {
        Self::load_path(&Self::path()?)
    }

    pub fn load_path(path: &Path) -> Option<Self> {
        log::debug!("Loading window state from {:?}", path);

        let data = match fs::read(path) {
            Ok(data) => data,
            Err(err) => {
                log::debug!("Could not load window state from {path:?}: {err}");
                return None;
            }
        };

        match serde_json::from_slice(&data) {
            Ok(state) => Some(state),
            Err(err) => {
                log::error!(
                    "Window state file is broken. Remove {path:?} to solve this error: {err}",
                );
                None
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let Some(path) = Self::path() else {
            return Ok(());
        };
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).with_context(|| {
                format!("Could not create directory for window state: {:?}", dir)
            })?;
        }
        self.save_path(&path)
    }

    pub fn save_path(&self, path: &Path) -> Result<()> {
        let data = serde_json::to_string(&self)
            .with_context(|| format!("Could not serialize window state file {path:?}"))?;
        log::debug!("Saving window state to {:?}", path);
        Ok(fs::write(path, data)?)
    }
}
