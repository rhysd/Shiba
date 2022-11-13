use crate::cli::Options;
use crate::config::Search as SearchConfig;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum KeyAction {
    Forward,
    Back,
    Reload,
    OpenFile,
    OpenDir,
    ScrollDown,
    ScrollUp,
    ScrollLeft,
    ScrollRight,
    ScrollPageDown,
    ScrollPageUp,
    ScrollTop,
    ScrollBottom,
    Search,
    NextSearch,
    PrevSearch,
    Quit,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageToRenderer<'a> {
    Content {
        content: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        offset: Option<usize>,
    },
    Config {
        keymaps: &'a HashMap<String, KeyAction>,
        search: &'a SearchConfig,
    },
    Search,
    SearchNext,
    SearchPrevious,
    Debug,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageFromRenderer {
    Init,
    Reload,
    FileDialog,
    DirDialog,
    Forward,
    Back,
    Quit,
    Error { message: String },
}

#[derive(Debug)]
pub enum UserEvent {
    IpcMessage(MessageFromRenderer),
    FileDrop(PathBuf),
    WatchedFilesChanged(Vec<PathBuf>),
    OpenLocalPath(PathBuf),
    OpenExternalLink(String),
    Error(Error),
}

#[derive(Debug)]
pub enum MenuItem {
    Quit,
    Forward,
    Back,
    Reload,
    OpenFile,
    WatchDir,
    Search,
    SearchNext,
    SearchPrevious,
}

pub trait MenuItems {
    type ItemId: fmt::Debug;
    fn item_from_id(&self, id: Self::ItemId) -> Result<MenuItem>;
}

pub trait RawMessageWriter {
    fn write_to(self, writer: impl fmt::Write) -> std::result::Result<(), fmt::Error>;
}

pub trait Renderer: Sized {
    type EventLoop;
    type Menu: MenuItems;

    fn open(options: &Options, event_loop: &Self::EventLoop) -> Result<Self>;
    fn menu(&self) -> &Self::Menu;
    fn send_message(&self, message: MessageToRenderer<'_>) -> Result<()>;
    fn send_message_raw(&self, writer: impl RawMessageWriter) -> Result<()>;
    fn set_title(&self, title: &str);
}
