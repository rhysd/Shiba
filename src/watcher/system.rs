use super::{find_watch_path_fallback, should_watch_event, PathFilter, Watcher};
use crate::renderer::{Event, EventSender};
use anyhow::{Context as _, Result};
use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::Path;

impl Watcher for RecommendedWatcher {
    fn new<S: EventSender>(sender: S, mut filter: PathFilter) -> Result<Self> {
        let watcher = recommended_watcher(move |res: notify::Result<notify::Event>| match res {
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
        Ok(watcher)
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
        log::debug!("Watching path {:?} with mode={:?}", path, mode);
        <Self as NotifyWatcher>::watch(self, path, mode)
            .context("Error while starting to watch a path")
    }
}

pub type SystemWatcher = RecommendedWatcher;
