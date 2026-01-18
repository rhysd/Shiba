use crate::config::Config;
use crate::persistent::{RecentFiles, RecentFilesOwned};
use anyhow::Result;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

pub struct History {
    max_items: usize,
    index: usize,
    items: VecDeque<PathBuf>,
}

impl History {
    pub fn load(config: &Config) -> Self {
        let max_items = config.preview().recent_files;
        if max_items > 0 {
            if let Some(mut recent) = config.data_dir().load::<RecentFilesOwned>() {
                recent.paths.truncate(max_items);
                log::debug!("Loaded {} paths as recent files history", recent.paths.len());
                return Self {
                    max_items,
                    index: recent.paths.len() - 1,
                    items: VecDeque::from(recent.paths),
                };
            }
        }

        Self { max_items, index: 0, items: VecDeque::new() }
    }

    pub fn push(&mut self, item: PathBuf) {
        if self.max_items == 0 {
            return;
        }

        if let Some(current) = self.current() {
            if current == &item {
                return; // Do not push the same path repeatedly
            }
        } else {
            log::debug!("Push first history item: {:?}", item);
            self.items.push_back(item);
            return;
        }

        if self.items.len() == self.max_items {
            self.items.pop_front();
            self.index = self.index.saturating_sub(1);
        }

        if self.index + 1 < self.items.len() {
            self.items.truncate(self.index + 1);
        }

        self.index += 1;
        log::debug!(
            "Push new history item at index {} (size={}): {:?}",
            self.index,
            self.items.len(),
            item,
        );
        self.items.push_back(item);
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
        self.items.get(self.index + 1)
    }

    pub fn prev(&self) -> Option<&PathBuf> {
        self.items.get(self.index.checked_sub(1)?)
    }

    pub fn current(&self) -> Option<&PathBuf> {
        self.items.get(self.index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ Path> {
        self.items.iter().map(PathBuf::as_path)
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        if self.max_items == 0 {
            return Ok(());
        }

        let mut seen = HashSet::new();
        let mut paths = vec![];
        for path in self.items.iter().map(|p| p.as_path()) {
            if seen.insert(path) {
                paths.push(path);
            }
        }

        log::debug!("Saving {} paths as recent files history", paths.len());
        config.data_dir().save(&RecentFiles { paths })
    }
}
