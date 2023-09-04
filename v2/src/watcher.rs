use crate::config::{FileExtensions, Watch as Config};
use crate::renderer::{EventChannel, EventLoop, UserEvent};
use anyhow::{Context as _, Result};
use notify::event::{CreateKind, DataChange, EventKind as WatchEventKind, ModifyKind};
use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

// Watching file paths don't work. Instead we need to watch their parent directories.
// This is a limitation only on Linux.
// https://github.com/notify-rs/notify/issues/531
#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::collections::HashSet;
    use std::ffi::OsString;

    #[derive(Default, Debug)]
    pub struct WatchingPaths {
        dirs: HashSet<PathBuf>,
        files: HashMap<PathBuf, HashSet<OsString>>,
    }

    impl WatchingPaths {
        pub fn watched_path<'a>(
            &mut self,
            path: &'a Path,
        ) -> Result<Option<(&'a Path, RecursiveMode)>> {
            assert!(path.is_absolute(), "Path to watch must be an absolute path: {:?}", path);
            if path.is_dir() {
                self.files.retain(|p, _| !p.starts_with(path));
                self.dirs.insert(path.into());
                log::debug!("Watching the existing directory recursively: {:?}", path);
                Ok(Some((path, RecursiveMode::Recursive)))
            } else if path.exists() {
                let (parent, file) = (path.parent().unwrap(), path.file_name().unwrap());
                if self.dirs.iter().any(|dir| parent.starts_with(dir)) {
                    log::debug!(
                        "Some parent directory is already watched: {:?}/{:?}",
                        parent,
                        file,
                    );
                    return Ok(None); // Its parent directory is already watched
                }

                log::debug!(
                    "Watching the parent directory non-recursively for file {:?}: {:?}",
                    file,
                    parent,
                );
                let file_names = self.files.entry(parent.into()).or_insert_with(HashSet::new);
                file_names.insert(file.to_os_string());
                Ok(Some((parent, RecursiveMode::NonRecursive)))
            } else if Some(parent) = path.ancestors().skip(1).find(|p| p.is_dir()) {
                log::warn!("Path {:?} does not exist. Watching the existing ancestor directory {:?} recursively instead", path, parent);
                Ok(Some((parent, RecursiveMode::Recursive)))
            } else {
                anyhow::bail!("Could not watch path {path:?} since all ancestors don't exist", path)
            }
        }

        pub fn should_retain(&self, path: &Path) -> bool {
            let (Some(parent), Some(file_name)) = (path.parent(), path.file_name()) else {
                return true;
            };
            let Some(file_names) = self.files.get(parent) else {
                return true;
            };
            file_names.contains(file_name)
        }
    }
}

pub struct SystemWatcher {
    inner: RecommendedWatcher,
    #[cfg(target_os = "linux")]
    watching: Arc<Mutex<linux::WatchingPaths>>,
}

#[cfg(not(target_os = "linux"))]
fn find_path_to_watch(path: &Path) -> Result<(&Path, RecursiveMode)> {
    if path.is_dir() {
        Ok((path, RecursiveMode::Recursive))
    } else if path.exists() {
        Ok((path, RecursiveMode::NonRecursive))
    } else if let Some(parent) = path.ancestors().skip(1).find(|p| p.is_dir()) {
        log::warn!("Path {:?} does not exist. Watching the existing ancestor directory {:?} recursively instead", path, parent);
        Ok((parent, RecursiveMode::Recursive))
    } else {
        anyhow::bail!("Could not watch path {:?} since it and all its parents don't exist", path)
    }
}

impl Watcher for SystemWatcher {
    fn new<E: EventLoop>(event_loop: &E, mut filter: PathFilter) -> Result<Self> {
        let channel = event_loop.create_channel();
        #[cfg(target_os = "linux")]
        let watching = Arc::new(Mutex::new(linux::WatchingPaths::default()));
        let inner = {
            #[cfg(target_os = "linux")]
            let watching = watching.clone();
            recommended_watcher(move |res: notify::Result<notify::Event>| match res {
                Ok(event) => match event.kind {
                    WatchEventKind::Create(CreateKind::File)
                    | WatchEventKind::Modify(
                        ModifyKind::Data(DataChange::Content | DataChange::Any) | ModifyKind::Any,
                    ) => {
                        log::debug!("Caught filesystem event: {:?}", event);

                        // XXX: Watcher sends the event at the first file-changed event durating debounce throttle.
                        // If the content is updated multiple times within the duration, only the first change is
                        // reflected to the preview.

                        let mut paths = event.paths;
                        #[cfg(not(target_os = "linux"))]
                        paths.retain(|p| filter.should_retain(p));
                        #[cfg(target_os = "linux")]
                        {
                            let watching = watching.lock().unwrap();
                            paths.retain(|p| filter.should_retain(p) && watching.should_retain(p));
                        }

                        if !paths.is_empty() {
                            log::debug!("Files change event from watcher: {:?}", paths);
                            channel.send_event(UserEvent::WatchedFilesChanged(paths));
                        }

                        filter.cleanup_debouncer();
                    }
                    _ => log::debug!("Ignored filesystem event: {:?}", event),
                },
                Err(err) => {
                    log::error!("Error on watching file changes: {}", err);
                    channel.send_event(UserEvent::Error(err.into()));
                }
            })?
        };
        Ok(SystemWatcher {
            inner,
            #[cfg(target_os = "linux")]
            watching,
        })
    }

    fn watch(&mut self, path: &Path) -> Result<()> {
        #[rustfmt::skip] // https://github.com/rust-lang/rustfmt/issues/5901
        #[cfg(target_os = "linux")]
        let Some((path, mode)) = self.watching.lock().unwrap().watched_path(path)? else {
            return Ok(());
        };
        #[cfg(not(target_os = "linux"))]
        let (path, mode) = find_path_to_watch(path)?;
        log::debug!("Watching path {:?} with mode={:?}", path, mode);
        self.inner.watch(path, mode).context("Error while starting to watch a path")
    }
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
