use super::{PathFilter, Watcher, find_watch_path_fallback, should_watch_event};
use crate::renderer::{Event, EventSender};
use anyhow::{Context as _, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher, recommended_watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct SystemWatcher {
    inner: RecommendedWatcher,
    watching: HashMap<PathBuf, RecursiveMode>,
}

impl Watcher for SystemWatcher {
    fn new<S: EventSender>(sender: S, mut filter: PathFilter) -> Result<Self> {
        let inner = recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) if should_watch_event(event.kind) => {
                log::debug!("Caught filesystem event: {:?}", event);

                let mut paths = event.paths;
                paths.retain(|p| filter.should_retain(p));

                if !paths.is_empty() {
                    log::debug!("Files change event from watcher: {:?}", paths);
                    sender.send(Event::WatchedFilesChanged(paths));
                }

                filter.cleanup_debouncer();
            }
            Ok(event) => log::debug!("Ignored filesystem event: {:?}", event),
            Err(err) => {
                log::error!("Error on watching file changes: {}", err);
                sender.send(Event::Error(err.into()));
            }
        })?;

        Ok(Self { inner, watching: HashMap::new() })
    }

    fn watch(&mut self, path: &Path) -> Result<()> {
        let (path, mode) = match path.metadata() {
            Ok(m) if m.is_dir() => (path, RecursiveMode::Recursive),
            Ok(_) => (path, RecursiveMode::NonRecursive),
            Err(err) => {
                log::debug!("Could not get metadata of {:?}: {}", path, err);
                (find_watch_path_fallback(path)?, RecursiveMode::Recursive)
            }
        };

        match self.watching.get_mut(path) {
            Some(prev) if mode == *prev => {
                log::debug!("Skip watching {:?} because it is already being watched", path);
                Ok(())
            }
            Some(prev) => {
                log::debug!("Changing watch mode for path {:?}: {:?} -> {:?}", path, prev, mode);
                self.inner.unwatch(path).context("Error while unwatching a path")?;
                *prev = mode;
                self.inner.watch(path, mode).context("Error while re-watching a path")
            }
            None => {
                log::debug!("Watching path {:?} with mode={:?}", path, mode);
                self.inner.watch(path, mode).context("Error while starting to watch a path")
            }
        }
    }
}
