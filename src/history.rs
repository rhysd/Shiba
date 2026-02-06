use crate::config::Config;
use crate::persistent::{HistoryData, HistoryDataOwned};
use crate::renderer::{MessageToRenderer, Renderer};
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
        let max_items = config.preview().history_size;
        if max_items > 0
            && let Some(mut data) = config.data_dir().load::<HistoryDataOwned>()
        {
            data.paths.truncate(max_items);
            log::debug!("Loaded {} paths from persistent history data", data.paths.len());
            let index = data.paths.len() - 1;
            return Self { max_items, index, items: data.paths };
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

    pub fn save(&self, config: &Config) -> Result<()> {
        if self.max_items == 0 {
            return Ok(());
        }

        log::debug!("Saving {} paths as persistent history data", self.items.len());
        let data = HistoryData { paths: &self.items };
        config.data_dir().save(&data)
    }

    pub fn send_paths<R: Renderer>(&self, renderer: &R) -> Result<()> {
        log::debug!("Send {} history paths to renderer", self.items.len());
        renderer.send_message(MessageToRenderer::History { paths: &self.items })
    }
}
