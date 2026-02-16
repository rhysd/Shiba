use crate::config::Config;
use crate::persistent::PersistentData;
use crate::renderer::{MessageToRenderer, Renderer};
use anyhow::Result;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DATA_FILE_NAME: &str = "history.json";

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Forward,
    Back,
    // Note: Adding Direction::Top and Direction::Bottom may be useful
}

pub struct History {
    max_items: usize,
    index: usize,
    items: IndexSet<PathBuf>,
}

impl History {
    pub fn new(max_items: usize) -> Self {
        Self { max_items, index: 0, items: IndexSet::new() }
    }

    pub fn with_paths(mut items: IndexSet<PathBuf>, max_items: usize) -> Self {
        items.truncate(max_items);
        log::debug!("Loaded {} paths from persistent history data", items.len());
        let index = items.len().saturating_sub(1);
        Self { max_items, index, items }
    }

    pub fn load(config: &Config) -> Self {
        #[derive(Deserialize)]
        struct Data {
            paths: IndexSet<PathBuf>,
        }
        impl PersistentData for Data {
            const FILE: &str = DATA_FILE_NAME;
        }

        let max_items = config.preview().history_size;
        if max_items > 0
            && let Some(data) = config.data_dir().load::<Data>()
        {
            Self::with_paths(data.paths, max_items)
        } else {
            Self::new(max_items)
        }
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

    pub fn current(&self) -> Option<&Path> {
        Some(self.items.get_index(self.index)?)
    }

    fn forward(&mut self) -> Option<&Path> {
        let path = self.items.get_index(self.index + 1)?;
        self.index += 1;
        Some(path)
    }

    fn back(&mut self) -> Option<&Path> {
        let idx = self.index.checked_sub(1)?;
        let path = self.items.get_index(idx)?;
        self.index = idx;
        Some(path)
    }

    pub fn navigate(&mut self, dir: Direction) -> Option<&Path> {
        match dir {
            Direction::Forward => self.forward(),
            Direction::Back => self.back(),
        }
    }

    pub fn delete(&mut self, dir: Direction) -> Option<&Path> {
        let removed = self.items.shift_remove_index(self.index)?;
        log::debug!("Deleted path from history: {removed:?}");
        match dir {
            Direction::Forward => self.current(),
            Direction::Back => self.back(),
        }
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        #[derive(Serialize)]
        struct Data<'a> {
            paths: &'a IndexSet<PathBuf>,
        }
        impl PersistentData for Data<'_> {
            const FILE: &'static str = DATA_FILE_NAME;
        }

        if self.max_items == 0 {
            return Ok(());
        }

        log::debug!("Saving {} paths as persistent history data", self.items.len());
        let data = Data { paths: &self.items };
        config.data_dir().save(&data)
    }

    pub fn send_paths<R: Renderer>(&self, renderer: &R) -> Result<()> {
        log::debug!("Send {} history paths to renderer", self.items.len());
        renderer.send_message(MessageToRenderer::History { paths: &self.items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;
    use crate::test::TestRenderer;

    #[test]
    fn send_paths_to_renderer() {
        let history = History::with_paths(["foo.txt".into(), "bar.txt".into()].into(), 10);
        let renderer = TestRenderer::default();
        history.send_paths(&renderer).unwrap();
        let msg = renderer.messages.take().pop().unwrap();
        let json: serde_json::Value = serde_json::from_str(&msg).unwrap();
        insta::assert_json_snapshot!(json);
    }

    #[test]
    fn load_save_persistent_data() {
        let dir = tempfile::tempdir().unwrap();
        let opts = Options {
            config_dir: Some(dir.path().to_path_buf()),
            data_dir: Some(dir.path().to_path_buf()),
            ..Default::default()
        };
        let config = Config::load(opts).unwrap();

        let paths = ["foo.txt".into(), "bar.txt".into()];
        let history = History::with_paths(paths.clone().into(), 10);
        history.save(&config).unwrap();
        assert!(dir.path().join(DATA_FILE_NAME).exists());

        let history = History::load(&config);
        let items: Vec<_> = history.items.into_iter().collect();
        assert_eq!(items, paths);
    }
}
