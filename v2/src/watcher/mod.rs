#[cfg(not(target_os = "linux"))]
mod system;
#[cfg(target_os = "linux")]
mod system_linux;

#[cfg(not(target_os = "linux"))]
pub use system::SystemWatcher;
#[cfg(target_os = "linux")]
pub use system_linux::SystemWatcher;

use crate::config::{FileExtensions, Watch as Config};
use crate::renderer::EventLoop;
use anyhow::Result;
use notify::event::{CreateKind, DataChange, EventKind, ModifyKind};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn find_watch_path_fallback(path: &Path) -> Result<&Path> {
    if let Some(parent) = path.ancestors().skip(1).find(|p| p.is_dir()) {
        log::warn!("Path {:?} does not exist. Watching the existing ancestor directory {:?} recursively instead", path, parent);
        Ok(parent)
    } else {
        anyhow::bail!("Could not watch path {:?} since its ancestors cannot be watched", path)
    }
}

fn should_watch_event(kind: EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(CreateKind::File)
            | EventKind::Modify(
                ModifyKind::Data(DataChange::Content | DataChange::Any) | ModifyKind::Any,
            ),
    )
}

pub struct PathFilter {
    extensions: FileExtensions,
    last_changed: HashMap<PathBuf, Instant>,
    debounce_throttle: Duration,
}

impl PathFilter {
    pub fn new(config: &Config) -> Self {
        let extensions = config.file_extensions().clone();
        let debounce_throttle = config.debounce_throttle();
        Self { extensions, last_changed: HashMap::new(), debounce_throttle }
    }

    // XXX: Watcher sends the event at the first file-changed event durating debounce throttle.
    // If the content is updated multiple times within the duration, only the first change is
    // reflected to the preview.
    fn debounce(&mut self, path: &Path) -> bool {
        let now = Instant::now();
        if let Some(last_changed) = self.last_changed.get_mut(path) {
            if now.duration_since(*last_changed) <= self.debounce_throttle {
                log::debug!("Debounced file-changed event for {:?}", path);
                return false;
            }
            *last_changed = now;
        } else {
            self.last_changed.insert(path.to_path_buf(), now);
        }
        true
    }

    fn should_retain(&mut self, path: &Path) -> bool {
        self.extensions.matches(path) && path.is_file() && self.debounce(path)
    }

    fn cleanup_debouncer(&mut self) {
        let before = self.last_changed.len();
        let now = Instant::now();
        self.last_changed
            .retain(|_, last_changed| now.duration_since(*last_changed) <= self.debounce_throttle);
        let expired = before - self.last_changed.len();
        if expired > 0 {
            log::debug!("Cleanup file-changed event debouncer. {} entries were expired", expired);
        }
    }
}

pub trait Watcher: Sized {
    fn new<E: EventLoop>(event_loop: &E, filter: PathFilter) -> Result<Self>;
    fn watch(&mut self, path: &Path) -> Result<()>;
}

pub struct NopWatcher;

impl Watcher for NopWatcher {
    fn new<E: EventLoop>(_event_loop: &E, _filter: PathFilter) -> Result<Self> {
        Ok(Self)
    }
    fn watch(&mut self, _path: &Path) -> Result<()> {
        Ok(())
    }
}
