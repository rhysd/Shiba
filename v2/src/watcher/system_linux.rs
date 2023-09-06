use super::{find_watch_path_fallback, should_watch_event, PathFilter, Watcher};
use crate::renderer::{EventChannel, EventLoop, UserEvent};
use anyhow::{Context as _, Result};
use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// Watching file paths don't work. Instead we need to watch their parent directories.
// This is a limitation only on Linux.
// https://github.com/notify-rs/notify/issues/531

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
                log::debug!("Some parent directory is already watched: {:?}/{:?}", parent, file,);
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
        } else {
            Ok(Some((find_watch_path_fallback(path)?, RecursiveMode::Recursive)))
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

pub struct SystemWatcher {
    inner: RecommendedWatcher,
    watching: Arc<Mutex<WatchingPaths>>,
}

impl Watcher for SystemWatcher {
    fn new<E: EventLoop>(event_loop: &E, mut filter: PathFilter) -> Result<Self> {
        let channel = event_loop.create_channel();
        let watching = Arc::new(Mutex::new(WatchingPaths::default()));
        let inner = {
            let watching = watching.clone();
            recommended_watcher(move |res: notify::Result<notify::Event>| match res {
                Ok(event) if should_watch_event(event.kind) => {
                    log::debug!("Caught filesystem event: {:?}", event);

                    let mut paths = event.paths;
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
                Ok(event) => log::debug!("Ignored filesystem event: {:?}", event),
                Err(err) => {
                    log::error!("Error on watching file changes: {}", err);
                    channel.send_event(UserEvent::Error(err.into()));
                }
            })?
        };
        Ok(Self { inner, watching })
    }

    fn watch(&mut self, path: &Path) -> Result<()> {
        #[rustfmt::skip] // https://github.com/rust-lang/rustfmt/issues/5901
        let Some((path, mode)) = self.watching.lock().unwrap().watched_path(path)? else {
            return Ok(());
        };
        log::debug!("Watching path {:?} with mode={:?}", path, mode);
        self.inner.watch(path, mode).context("Error while starting to watch a path")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn fullpath_fn() -> impl Fn(&str) -> PathBuf {
        let mut p = env::current_dir().unwrap();
        p.push("src/watcher/testdata");
        move |path| p.join(path)
    }

    #[test]
    fn paths_watch_files_then_dirs() {
        let fullpath = fullpath_fn();
        let foo_bar_b = fullpath("foo/bar/b.txt");
        let foo_a = fullpath("foo/a.txt");
        let foo_bar = fullpath("foo/bar");
        let foo = fullpath("foo");

        let mut watching = WatchingPaths::default();

        let (p, m) = watching.watched_path(&foo_bar_b).unwrap().unwrap();
        assert_eq!(m, RecursiveMode::NonRecursive);
        assert_eq!(p, &foo_bar);

        let (p, m) = watching.watched_path(&foo_a).unwrap().unwrap();
        assert_eq!(m, RecursiveMode::NonRecursive);
        assert_eq!(p, &foo);

        assert!(watching.files.contains_key(&foo_bar));
        assert!(watching.files.contains_key(&foo));
        assert!(watching.dirs.is_empty());

        assert!(watching.should_retain(&foo_bar_b));
        assert!(watching.should_retain(&foo_a));

        let (p, m) = watching.watched_path(&foo_bar).unwrap().unwrap();
        assert_eq!(m, RecursiveMode::Recursive);
        assert_eq!(p, &foo_bar);

        // Since foo/bar contains foo/bar/b.txt, foo/bar/b.txt is no longer watched
        assert!(!watching.files.contains_key(&foo_bar));
        assert!(watching.files.contains_key(&foo));
        assert!(watching.dirs.contains(&foo_bar));

        assert!(watching.should_retain(&foo_bar_b));
        assert!(watching.should_retain(&foo_a));

        let (p, m) = watching.watched_path(&foo).unwrap().unwrap();
        assert_eq!(m, RecursiveMode::Recursive);
        assert_eq!(p, &foo);

        // Since foo contains foo/a.txt, foo/a.txt is no longer watched
        assert!(watching.files.is_empty());
        assert!(watching.dirs.contains(&foo_bar));
        assert!(watching.dirs.contains(&foo));

        assert!(watching.should_retain(&foo_bar_b));
        assert!(watching.should_retain(&foo_a));

        let mut watching = WatchingPaths::default();
        watching.watched_path(&foo_bar_b).unwrap().unwrap();
        watching.watched_path(&foo_a).unwrap().unwrap();
        watching.watched_path(&foo).unwrap().unwrap();

        // Since foo contains both, only foo is watched as the result
        assert!(watching.files.is_empty());
        assert!(watching.dirs.contains(&foo));

        assert!(watching.should_retain(&foo_bar_b));
        assert!(watching.should_retain(&foo_a));
    }

    #[test]
    fn paths_watch_dirs_then_files() {
        let fullpath = fullpath_fn();
        let foo_bar_b = fullpath("foo/bar/b.txt");
        let foo_a = fullpath("foo/a.txt");
        let foo_bar = fullpath("foo/bar");
        let foo = fullpath("foo");

        let mut watching = WatchingPaths::default();

        let (p, m) = watching.watched_path(&foo_bar).unwrap().unwrap();
        assert_eq!(m, RecursiveMode::Recursive);
        assert_eq!(p, &foo_bar);

        // foo/bar/b.txt is not watched because foo/bar is already watched
        assert_eq!(watching.watched_path(&foo_bar_b).unwrap(), None);

        // foo/a.txt is watched because foo/bar doesn't contain it
        let (p, m) = watching.watched_path(&foo_a).unwrap().unwrap();
        assert_eq!(m, RecursiveMode::NonRecursive);
        assert_eq!(p, &foo);

        assert!(!watching.files.contains_key(&foo_bar));
        assert!(watching.files.contains_key(&foo));
        assert!(watching.should_retain(&foo_bar_b));
        assert!(watching.should_retain(&foo_a));

        let mut watching = WatchingPaths::default();

        let (p, m) = watching.watched_path(&foo).unwrap().unwrap();
        assert_eq!(m, RecursiveMode::Recursive);
        assert_eq!(p, &foo);

        // foo/bar/b.txt and foo/a.txt are not watched because foo/bar is already watched
        assert_eq!(watching.watched_path(&foo_bar_b).unwrap(), None);
        assert_eq!(watching.watched_path(&foo_a).unwrap(), None);

        assert!(watching.files.is_empty());
        assert!(watching.should_retain(&foo_bar_b));
        assert!(watching.should_retain(&foo_a));
    }

    #[test]
    fn paths_should_retain() {
        let fullpath = fullpath_fn();
        let foo_bar_b = fullpath("foo/bar/b.txt");
        let foo_a = fullpath("foo/a.txt");
        let foo_bar_c = fullpath("foo/bar/c.txt");
        let foo_d = fullpath("foo/d.txt");

        let tests = [
            (vec![], (true, true)),
            (vec![&foo_a], (true, false)),
            (vec![&foo_bar_b], (false, true)),
            (vec![&foo_a, &foo_bar_b], (false, false)),
        ];

        for (paths, (expect_c, expect_d)) in tests {
            let mut watching = WatchingPaths::default();
            for path in paths.iter() {
                watching.watched_path(path).unwrap().unwrap();
            }
            assert_eq!(
                watching.should_retain(&foo_bar_c),
                expect_c,
                "{:?} and {:?}",
                paths,
                watching,
            );
            assert_eq!(watching.should_retain(&foo_d), expect_d, "{:?} and {:?}", paths, watching);
        }
    }

    #[test]
    fn paths_watch_not_existing_paths() {
        let fullpath = fullpath_fn();
        let foo_bar = fullpath("foo/bar");
        let foo = fullpath("foo");

        let file = fullpath("foo/bar/not-existing.txt");
        let dir = fullpath("foo/not-existing/");
        let nest = fullpath("foo/not-existing/not-existing.txt");

        let mut watching = WatchingPaths::default();

        let (p, m) = watching.watched_path(&file).unwrap().unwrap();
        assert_eq!(m, RecursiveMode::Recursive);
        assert_eq!(p, &foo_bar);

        let (p, m) = watching.watched_path(&dir).unwrap().unwrap();
        assert_eq!(m, RecursiveMode::Recursive);
        assert_eq!(p, &foo);

        let (p, m) = watching.watched_path(&nest).unwrap().unwrap();
        assert_eq!(m, RecursiveMode::Recursive);
        assert_eq!(p, &foo);

        // Error case cannot be tested because '/' always exists
    }
}
