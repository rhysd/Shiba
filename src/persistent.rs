use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub trait PersistentData {
    const FILE: &str;
}

#[derive(Debug)]
pub struct DataDir {
    path: Option<PathBuf>,
}

impl Default for DataDir {
    fn default() -> Self {
        fn path() -> Option<PathBuf> {
            let mut dir = dirs::data_dir()?;
            dir.push("Shiba");
            fs::create_dir_all(&dir).ok()?;
            log::debug!("Default data directory: {dir:?}");
            Some(dir)
        }
        Self { path: path() }
    }
}

impl DataDir {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        let path = dir.is_dir().then_some(dir);
        log::debug!("Custom data directory path: {path:?}");
        Self { path }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Test(bool);

    impl PersistentData for Test {
        const FILE: &str = "test.json";
    }

    #[test]
    fn save_and_load() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = DataDir::new(tmp.path());
        assert_eq!(dir.path(), Some(tmp.path()));
        let expected = Test(true);
        dir.save(&expected).unwrap();
        let file = tmp.path().join("test.json");
        assert!(file.exists(), "path={file:?}");
        let actual = dir.load().unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn no_data_dir() {
        let dir = DataDir::new("this-directory-does-not-exist");
        assert!(dir.path.is_none());
        assert_eq!(dir.load::<Test>(), None);
        dir.save(&Test(true)).unwrap(); // Does nothing and it's not an error
    }

    #[test]
    fn default_dir() {
        let dir = DataDir::default();
        let Some(path) = dir.path() else {
            return; // Data directory was not found
        };
        assert!(path.ends_with("Shiba"), "path={path:?}");
    }
}
