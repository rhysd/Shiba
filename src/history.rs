use crate::config::Config;
use crate::persistent::{RecentFiles, RecentFilesOwned};
use anyhow::Result;
use indexmap::IndexSet;
use std::path::{Path, PathBuf};

pub struct History {
    max_items: usize,
    index: usize,
    items: IndexSet<PathBuf>,
}

impl History {
    pub fn new(max_items: usize) -> Self {
        Self { max_items, index: 0, items: IndexSet::new() }
    }

    pub fn load(config: &Config) -> Self {
        let max_items = config.preview().recent_files;
        if max_items > 0
            && let Some(mut recent) = config.data_dir().load::<RecentFilesOwned>()
        {
            recent.paths.truncate(max_items);
            log::debug!("Loaded {} paths as recent files history", recent.paths.len());
            let index = recent.paths.len() - 1;
            return Self { max_items, index, items: recent.paths };
        }

        Self::new(max_items)
    }

    pub fn push(&mut self, item: PathBuf) {
        if self.max_items == 0 {
            return;
        }

        if self.items.shift_remove(&item) {
            // Move to the existing item instead of inserting a new item
            log::debug!("Move the existing item to top of history: {:?}", item);
        } else {
            if self.items.len() == self.max_items {
                self.items.shift_remove_index(0);
            }
            log::debug!("Push the new item to history (size={}): {:?}", self.items.len() + 1, item);
        }

        self.items.insert(item);
        self.index = self.items.len() - 1; // Reset index to put focus on the new item
    }

    pub fn current(&self) -> Option<&PathBuf> {
        self.items.get_index(self.index)
    }

    pub fn forward(&mut self) -> Option<&Path> {
        let path = self.items.get_index(self.index + 1)?;
        self.index += 1;
        Some(path)
    }

    pub fn back(&mut self) -> Option<&Path> {
        let idx = self.index.checked_sub(1)?;
        let path = self.items.get_index(idx)?;
        self.index = idx;
        Some(path)
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ Path> {
        self.items.iter().map(PathBuf::as_path)
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        if self.max_items == 0 {
            return Ok(());
        }

        log::debug!("Saving {} paths as recent files history", self.items.len());
        let data = RecentFiles { paths: &self.items };
        config.data_dir().save(&data)
    }
}
