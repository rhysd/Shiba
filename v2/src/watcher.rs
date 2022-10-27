use crate::renderer::UserEvent;
use anyhow::Result;
use notify::event::{CreateKind, DataChange, EventKind as WatchEventKind, ModifyKind};
use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::{Path, PathBuf};
use wry::application::event_loop::{EventLoop, EventLoopProxy};

pub struct PathFilter {
    extensions: Vec<String>,
}

impl PathFilter {
    pub fn new<I>(extensions: I) -> Self
    where
        I: IntoIterator,
        I::Item: ToString,
    {
        let extensions = extensions.into_iter().map(|x| x.to_string()).collect();
        Self { extensions }
    }

    pub fn filters(&self, path: &Path) -> bool {
        self.extensions
            .iter()
            .any(|e| path.extension().map(|ext| ext == e.as_str()).unwrap_or(false))
    }
}

pub trait WatchChannelCreator {
    type Channel: 'static + Send;

    fn create_channel(&self) -> Self::Channel;
    fn on_files_changed(chan: &Self::Channel, paths: Result<Vec<PathBuf>>);
}

impl WatchChannelCreator for EventLoop<UserEvent> {
    type Channel = EventLoopProxy<UserEvent>;

    fn create_channel(&self) -> Self::Channel {
        self.create_proxy()
    }

    fn on_files_changed(chan: &Self::Channel, paths: Result<Vec<PathBuf>>) {
        log::debug!("Files change event from watcher: {:?}", paths);
        let event = match paths {
            Ok(paths) => UserEvent::WatchedFilesChanged(paths),
            Err(err) => UserEvent::Error(err),
        };
        if let Err(err) = chan.send_event(event) {
            log::error!("Could not send the file change event {}", err);
        }
    }
}

pub trait Watcher: Sized {
    fn new<C: WatchChannelCreator>(creator: &C, filter: PathFilter) -> Result<Self>;
    fn watch(&mut self, path: &Path) -> Result<()>;
    fn unwatch(&mut self, path: &Path) -> Result<()>;
}

impl Watcher for RecommendedWatcher {
    fn new<C: WatchChannelCreator>(creator: &C, filter: PathFilter) -> Result<Self> {
        let channel = creator.create_channel();
        let watcher = recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => match event.kind {
                WatchEventKind::Create(CreateKind::File)
                | WatchEventKind::Modify(ModifyKind::Data(DataChange::Content)) => {
                    log::debug!("Caught filesystem event: {:?}", event.kind);
                    let mut paths = event.paths;
                    paths.retain(|p| filter.filters(p));
                    if !paths.is_empty() {
                        C::on_files_changed(&channel, Ok(paths));
                    }
                }
                _ => {}
            },
            Err(err) => C::on_files_changed(&channel, Err(err.into())),
        })?;
        Ok(watcher)
    }

    fn watch(&mut self, path: &Path) -> Result<()> {
        let mode =
            if path.is_dir() { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive };
        <RecommendedWatcher as NotifyWatcher>::watch(self, path, mode)?;
        Ok(())
    }

    fn unwatch(&mut self, path: &Path) -> Result<()> {
        <RecommendedWatcher as NotifyWatcher>::unwatch(self, path)?;
        Ok(())
    }
}
