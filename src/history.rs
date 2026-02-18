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

    pub fn with_items(mut items: IndexSet<PathBuf>, max_items: usize) -> Self {
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
            Self::with_items(data.paths, max_items)
        } else {
            Self::new(max_items)
        }
    }

    pub fn push(&mut self, item: PathBuf) {
        if self.max_items == 0 {
            return;
        }

        // Skip removing the last item and inserting it again
        if self.items.last() == Some(&item) {
            self.index = self.items.len() - 1;
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
    use crate::config::UserConfig;
    use crate::test::TestRenderer;

    #[track_caller]
    fn history_with(max_items: usize, paths: &[&str]) -> History {
        let mut history = History::new(max_items);
        for path in paths {
            history.push(path.into());
        }
        history
    }

    #[track_caller]
    fn assert_history(history: &History, expected: &[&str], current: Option<&str>) {
        let expected: Vec<_> = expected.iter().map(Path::new).collect();
        assert_eq!(history.items.as_slice(), expected.as_slice());

        let size = history.items.len();
        let max = history.max_items;
        assert!(size <= max, "size {size} is larger than max {max}");

        let current = current.map(Path::new);
        assert_eq!(history.current(), current);
    }

    #[test]
    fn send_paths_to_renderer() {
        let history = History::with_items(["foo.txt".into(), "bar.txt".into()].into(), 10);
        let renderer = TestRenderer::default();
        history.send_paths(&renderer).unwrap();
        let msg = renderer.messages.take().pop().unwrap();
        let json: serde_json::Value = serde_json::from_str(&msg).unwrap();
        insta::assert_json_snapshot!(json);
    }

    #[test]
    fn load_save_persistent_data() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::new(UserConfig::default(), dir.path(), dir.path());

        let paths = ["foo.txt".into(), "bar.txt".into()];
        let history = History::with_items(paths.clone().into(), 10);
        history.save(&config).unwrap();
        assert!(dir.path().join(DATA_FILE_NAME).exists());

        let history = History::load(&config);
        assert_eq!(history.items.as_slice(), &paths);
    }

    #[test]
    fn load_without_persistent_data_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::new(UserConfig::default(), dir.path(), dir.path());

        let history = History::load(&config);
        assert!(history.items.is_empty());
        assert_eq!(history.current(), None);
    }

    #[test]
    fn save_with_zero_max_items() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::new(UserConfig::default(), dir.path(), dir.path());
        History::new(0).save(&config).unwrap();
        assert!(!dir.path().join(DATA_FILE_NAME).exists());
    }

    #[test]
    fn push_items() {
        let mut history = history_with(3, &["a.txt", "b.txt"]);

        history.push("c.txt".into());
        assert_history(&history, &["a.txt", "b.txt", "c.txt"], Some("c.txt"));

        history.push("b.txt".into());
        assert_history(&history, &["a.txt", "c.txt", "b.txt"], Some("b.txt"));

        history.push("d.txt".into()); // Removes oldest item
        assert_history(&history, &["c.txt", "b.txt", "d.txt"], Some("d.txt"));

        history.push("a.txt".into());
        assert_history(&history, &["b.txt", "d.txt", "a.txt"], Some("a.txt"));

        history.push("a.txt".into()); // Does nothing
        assert_history(&history, &["b.txt", "d.txt", "a.txt"], Some("a.txt"));
    }

    #[test]
    fn navigate_forward_back() {
        let items = &["a.txt", "b.txt", "c.txt"];

        let mut history = history_with(3, items);
        assert_history(&history, items, Some("c.txt"));

        assert_eq!(history.navigate(Direction::Back).unwrap(), "b.txt");
        assert_history(&history, items, Some("b.txt"));

        assert_eq!(history.navigate(Direction::Back).unwrap(), "a.txt");
        assert_history(&history, items, Some("a.txt"));

        assert_eq!(history.navigate(Direction::Back), None);
        assert_history(&history, items, Some("a.txt"));

        assert_eq!(history.navigate(Direction::Forward).unwrap(), "b.txt");
        assert_history(&history, items, Some("b.txt"));

        assert_eq!(history.navigate(Direction::Forward).unwrap(), "c.txt");
        assert_history(&history, items, Some("c.txt"));

        assert_eq!(history.navigate(Direction::Forward), None);
        assert_history(&history, items, Some("c.txt"));
    }

    #[test]
    fn push_navigate_to_top() {
        let mut history = history_with(3, &["a.txt", "b.txt"]);

        assert_eq!(history.navigate(Direction::Back).unwrap(), "a.txt");
        assert_history(&history, &["a.txt", "b.txt"], Some("a.txt"));
        history.push("c.txt".into()); // Push new item
        assert_history(&history, &["a.txt", "b.txt", "c.txt"], Some("c.txt"));

        assert_eq!(history.navigate(Direction::Back).unwrap(), "b.txt");
        assert_history(&history, &["a.txt", "b.txt", "c.txt"], Some("b.txt"));
        history.push("a.txt".into()); // Push existing item
        assert_history(&history, &["b.txt", "c.txt", "a.txt"], Some("a.txt"));

        assert_eq!(history.navigate(Direction::Back).unwrap(), "c.txt");
        assert_history(&history, &["b.txt", "c.txt", "a.txt"], Some("c.txt"));
        history.push("a.txt".into()); // Push last item
        assert_history(&history, &["b.txt", "c.txt", "a.txt"], Some("a.txt"));
    }

    #[test]
    fn delete_back() {
        let mut history = history_with(3, &["a.txt", "b.txt", "c.txt"]);
        assert_history(&history, &["a.txt", "b.txt", "c.txt"], Some("c.txt"));

        assert_eq!(history.delete(Direction::Back).unwrap(), "b.txt");
        assert_history(&history, &["a.txt", "b.txt"], Some("b.txt"));

        assert_eq!(history.delete(Direction::Back).unwrap(), "a.txt");
        assert_history(&history, &["a.txt"], Some("a.txt"));

        assert_eq!(history.delete(Direction::Back), None);
        assert_history(&history, &[], None);

        assert_eq!(history.delete(Direction::Back), None);
        assert_history(&history, &[], None);
    }

    #[test]
    fn delete_forward() {
        let mut history = history_with(3, &["a.txt", "b.txt", "c.txt"]);
        while history.navigate(Direction::Back).is_some() {}
        assert_history(&history, &["a.txt", "b.txt", "c.txt"], Some("a.txt"));

        assert_eq!(history.delete(Direction::Forward).unwrap(), "b.txt");
        assert_history(&history, &["b.txt", "c.txt"], Some("b.txt"));

        assert_eq!(history.delete(Direction::Forward).unwrap(), "c.txt");
        assert_history(&history, &["c.txt"], Some("c.txt"));

        assert_eq!(history.delete(Direction::Forward), None);
        assert_history(&history, &[], None);

        assert_eq!(history.delete(Direction::Forward), None);
        assert_history(&history, &[], None);
    }

    #[test]
    fn with_items_truncates_to_max_items() {
        let history = History::with_items(
            ["a.txt".into(), "b.txt".into(), "c.txt".into(), "d.txt".into()].into(),
            2,
        );
        assert_history(&history, &["a.txt", "b.txt"], Some("b.txt"));
    }

    #[test]
    fn zero_max_items() {
        let mut history = History::new(0);
        assert_history(&history, &[], None);
        history.push("a.txt".into());
        assert_history(&history, &[], None);
        assert_eq!(history.navigate(Direction::Back), None);
        assert_history(&history, &[], None);
        assert_eq!(history.navigate(Direction::Forward), None);
        assert_history(&history, &[], None);
        assert_eq!(history.delete(Direction::Back), None);
        assert_history(&history, &[], None);
        assert_eq!(history.delete(Direction::Forward), None);
        assert_history(&history, &[], None);
    }

    #[test]
    fn empty_history() {
        let mut history = History::new(5);
        assert_history(&history, &[], None);
        assert_eq!(history.navigate(Direction::Back), None);
        assert_history(&history, &[], None);
        assert_eq!(history.navigate(Direction::Forward), None);
        assert_history(&history, &[], None);
        assert_eq!(history.delete(Direction::Back), None);
        assert_history(&history, &[], None);
        assert_eq!(history.delete(Direction::Forward), None);
        assert_history(&history, &[], None);
    }
}
