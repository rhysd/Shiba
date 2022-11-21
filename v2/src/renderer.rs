use crate::cli::Options;
use crate::config::{Config, Search as SearchConfig, SearchMatcher};
use crate::persistent::WindowState;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    NextSection,
    PrevSection,
    Outline,
    Quit,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageToRenderer<'a> {
    Config { keymaps: &'a HashMap<String, KeyAction>, search: &'a SearchConfig, theme: Theme },
    Search,
    SearchNext,
    SearchPrevious,
    Welcome,
    Outline,
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
    Search { query: String, index: Option<usize>, matcher: SearchMatcher },
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
    Outline,
    Print,
    ZoomIn,
    ZoomOut,
}

pub trait MenuItems {
    type ItemId: fmt::Debug;
    fn item_from_id(&self, id: Self::ItemId) -> Result<MenuItem>;
}

pub trait RawMessageWriter {
    type Output;
    fn write_to(self, writer: impl fmt::Write) -> std::result::Result<Self::Output, fmt::Error>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum Theme {
    Dark,
    Light,
}

pub trait Renderer: Sized {
    type EventLoop;
    type Menu: MenuItems;

    fn new(
        options: &Options,
        config: &Config,
        event_loop: &Self::EventLoop,
        window_state: Option<WindowState>,
    ) -> Result<Self>;
    fn menu(&self) -> &Self::Menu;
    fn send_message(&self, message: MessageToRenderer<'_>) -> Result<()>;
    fn send_message_raw<W: RawMessageWriter>(&self, writer: W) -> Result<W::Output>;
    fn set_title(&self, title: &str);
    fn window_state(&self) -> Option<WindowState>;
    fn theme(&self) -> Theme;
    fn show(&self);
    fn set_background_color(&self, rbga: (u8, u8, u8, u8)) -> Result<()>;
    fn print(&self) -> Result<()>;
    fn zoom(&self, scale: f64);
}
