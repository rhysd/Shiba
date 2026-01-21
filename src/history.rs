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
            let items = recent.paths.into_iter().collect();
            return Self { max_items, index, items };
        }

        Self::new(max_items)
    }

    pub fn push(&mut self, item: PathBuf) {
        if self.max_items == 0 {
            return;
        }

        if let Some(idx) = self.items.get_index_of(&item) {
            // Move to the existing item instead of inserting a new item
            log::debug!("Move to the index of existing history item {:?}: {}", item, idx);
            self.index = idx;
            return;
        }

        if self.items.len() == self.max_items {
            self.items.shift_remove_index(0);
        }

        log::debug!("Push new history item (size={}): {:?}", self.items.len(), item);
        self.items.insert(item);
        self.index = self.items.len() - 1; // Reset index to put focus on the new item
    }

    pub fn current(&self) -> Option<&PathBuf> {
        self.items.get_index(self.index)
    }

    pub fn forward(&mut self) {
        if self.index + 1 < self.items.len() {
            self.index += 1;
        }
    }

    pub fn back(&mut self) {
        if let Some(i) = self.index.checked_sub(1) {
            self.index = i;
        }
    }

    pub fn next(&self) -> Option<&PathBuf> {
        self.items.get_index(self.index + 1)
    }

    pub fn prev(&self) -> Option<&PathBuf> {
        self.items.get_index(self.index.checked_sub(1)?)
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ Path> {
        self.items.iter().map(PathBuf::as_path)
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        if self.max_items == 0 {
            return Ok(());
        }

        let paths: Vec<_> = self.items.iter().map(PathBuf::as_path).collect();
        log::debug!("Saving {} paths as recent files history", paths.len());
        config.data_dir().save(&RecentFiles { paths })
    }
}
