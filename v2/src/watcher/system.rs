use super::{find_watch_path_fallback, should_watch_event, PathFilter, Watcher};
use crate::renderer::{UserEvent, UserEventSender};
use anyhow::{Context as _, Result};
use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::Path;

fn find_path_to_watch(path: &Path) -> Result<(&Path, RecursiveMode)> {
    if path.is_dir() {
        Ok((path, RecursiveMode::Recursive))
    } else if path.exists() {
        Ok((path, RecursiveMode::NonRecursive))
    } else {
        Ok((find_watch_path_fallback(path)?, RecursiveMode::Recursive))
    }
}

impl Watcher for RecommendedWatcher {
    fn new<S: UserEventSender>(sender: S, mut filter: PathFilter) -> Result<Self> {
        let watcher = recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) if should_watch_event(event.kind) => {
                log::debug!("Caught filesystem event: {:?}", event);

                let mut paths = event.paths;
                paths.retain(|p| filter.should_retain(p));

                if !paths.is_empty() {
                    log::debug!("Files change event from watcher: {:?}", paths);
                    sender.send(UserEvent::WatchedFilesChanged(paths));
                }

                filter.cleanup_debouncer();
            }
            Ok(event) => log::debug!("Ignored filesystem event: {:?}", event),
            Err(err) => {
                log::error!("Error on watching file changes: {}", err);
                sender.send(UserEvent::Error(err.into()));
            }
        })?;
        Ok(watcher)
    }

    fn watch(&mut self, path: &Path) -> Result<()> {
        let (path, mode) = find_path_to_watch(path)?;
        log::debug!("Watching path {:?} with mode={:?}", path, mode);
        <Self as NotifyWatcher>::watch(self, path, mode)
            .context("Error while starting to watch a path")
    }
}

pub type SystemWatcher = RecommendedWatcher;
