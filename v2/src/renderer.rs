use crate::config::{Config, KeyAction, Search as SearchConfig, SearchMatcher};
use crate::persistent::WindowState;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageToRenderer<'a> {
    Config {
        keymaps: &'a HashMap<String, KeyAction>,
        search: &'a SearchConfig,
        theme: Theme,
        recent: Vec<&'a Path>,
    },
    Search,
    SearchNext,
    SearchPrevious,
    Welcome,
    Outline,
    NewFile {
        path: &'a Path,
    },
    History,
    Help,
    Zoom {
        percent: u16,
    },
    Reload,
    Debug,
    AlwaysOnTop {
        pinned: bool,
    },
}

#[derive(Clone, Copy, Deserialize, Debug)]
pub enum Zoom {
    In,
    Out,
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
    OpenFile { path: String },
    Zoom { zoom: Zoom },
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

#[derive(Clone, Copy, Debug)]
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
    History,
    Help,
    OpenRepo,
    ToggleAlwaysOnTop,
}

pub trait MenuItems {
    type ItemId: fmt::Debug;
    fn item_from_id(&self, id: Self::ItemId) -> Result<MenuItem>;
}

pub trait RawMessageWriter {
    type Output;
    fn write_to(self, writer: impl io::Write) -> io::Result<Self::Output>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum Theme {
    Dark,
    Light,
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ZoomLevel(u8);

impl ZoomLevel {
    const MAX: u8 = 14;

    // Following the same zoom factors in Chrome
    pub fn factor(self) -> f64 {
        match self.0 {
            0 => 0.25,
            1 => 0.33,
            2 => 0.50,
            3 => 0.67,
            4 => 0.75,
            5 => 0.80,
            6 => 0.90,
            7 => 1.00,
            8 => 1.10,
            9 => 1.25,
            10 => 1.50,
            11 => 1.75,
            12 => 2.00,
            13 => 2.50,
            14 => 3.00,
            _ => unreachable!("Invalid zoom level {:?}", self.0),
        }
    }

    pub fn percent(self) -> u16 {
        (self.factor() * 100.0) as u16
    }

    pub fn zoom_in(self) -> Option<Self> {
        (self.0 < Self::MAX).then_some(Self(self.0 + 1))
    }

    pub fn zoom_out(self) -> Option<Self> {
        self.0.checked_sub(1).map(Self)
    }
}

impl Default for ZoomLevel {
    fn default() -> Self {
        Self(7) // Zoom factor 1.0
    }
}

#[derive(Debug)]
pub enum AppControl {
    Continue,
    Exit,
}

pub trait App<M: MenuItems> {
    fn handle_user_event(&mut self, event: UserEvent) -> Result<AppControl>;
    fn handle_menu_event(&mut self, id: M::ItemId) -> Result<AppControl>;
    fn handle_exit(&self) -> Result<()>;
}

pub trait EventChannel: 'static + Send {
    fn send_event(&self, event: UserEvent);
}

pub trait EventLoop {
    type Channel: EventChannel;
    type Menu: MenuItems;
    fn create_channel(&self) -> Self::Channel;
    fn start<A>(self, app: A) -> !
    where
        A: App<Self::Menu> + 'static;
}

pub trait Renderer: Sized {
    type EventLoop: EventLoop;
    type Menu: MenuItems;

    fn new(
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
    fn zoom(&mut self, level: ZoomLevel);
    fn zoom_level(&self) -> ZoomLevel;
    fn set_always_on_top(&mut self, enabled: bool);
    fn always_on_top(&self) -> bool;
}
