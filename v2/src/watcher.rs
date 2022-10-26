use crate::renderer::UserEvent;
use anyhow::Result;
use notify::event::{CreateKind, DataChange, EventKind as WatchEventKind, ModifyKind};
use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::{Path, PathBuf};
use wry::application::event_loop::{EventLoop, EventLoopProxy};

// TODO: Change EventLoop<UserEvent> to EventLoop<Result<UserEvent>> to handle errors by the event loop

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
    type Channel;

    fn create_channel(&self) -> Self::Channel;
    fn send_changed_paths(chan: &Self::Channel, paths: Vec<PathBuf>);
}

impl WatchChannelCreator for EventLoop<UserEvent> {
    type Channel = EventLoopProxy<UserEvent>;

    fn create_channel(&self) -> Self::Channel {
        self.create_proxy()
    }

    fn send_changed_paths(chan: &Self::Channel, paths: Vec<PathBuf>) {
        log::debug!("Files change event from watcher: {:?}", paths);
        if let Err(err) = chan.send_event(UserEvent::WatchedFilesChanged(paths)) {
            log::error!("Could not send the file change event {}", err);
        }
    }
}

pub trait Watcher: Sized {
    type ChannelCreator: WatchChannelCreator;

    fn new(creator: &Self::ChannelCreator, filter: PathFilter) -> Result<Self>;
    fn watch(&mut self, path: &Path) -> Result<()>;
    fn unwatch(&mut self, path: &Path) -> Result<()>;
}

impl Watcher for RecommendedWatcher {
    type ChannelCreator = EventLoop<UserEvent>;

    fn new(creator: &Self::ChannelCreator, filter: PathFilter) -> Result<Self> {
        let channel = creator.create_channel();
        let watcher = recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => match event.kind {
                WatchEventKind::Create(CreateKind::File)
                | WatchEventKind::Modify(ModifyKind::Data(DataChange::Content)) => {
                    log::debug!("Caught filesystem event: {:?}", event.kind);
                    let mut paths = event.paths;
                    paths.retain(|p| filter.filters(p));
                    if !paths.is_empty() {
                        EventLoop::send_changed_paths(&channel, paths);
                    }
                }
                _ => {}
            },
            Err(e) => log::error!("Could not watch filesystem event: {}", e),
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
