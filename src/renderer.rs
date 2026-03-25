use crate::config::{Config, KeyAction, Search as SearchConfig, SearchMatcher};
use crate::persistent::PersistentData;
use anyhow::{Error, Result};
use indexmap::IndexSet;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WindowState {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub fullscreen: bool,
    pub zoom_level: ZoomLevel,
    pub always_on_top: bool,
    pub maximized: bool,
}

impl PersistentData for WindowState {
    const FILE: &'static str = "window.json";
}

#[derive(Clone, Copy, Default, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WindowAppearance {
    pub title: bool,
    pub vibrancy: bool,
    pub scroll_bar: bool,
    pub border_top: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScrollRequest<'a> {
    Fragment(&'a str),
    Heading(usize),
}

#[derive(Serialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageToWindow<'a> {
    Config {
        keymaps: &'a HashMap<String, KeyAction>,
        search: &'a SearchConfig,
        home: Option<&'a Path>,
        window: WindowAppearance,
    },
    Path {
        path: &'a Path,
    },
    Search,
    SearchNext,
    SearchPrevious,
    Welcome,
    Outline,
    History {
        paths: &'a IndexSet<PathBuf>,
    },
    Help,
    Zoomed {
        percent: u16,
    },
    Reload,
    AlwaysOnTop {
        pinned: bool,
    },
    // TODO: Ideally the information about initial scrolling should be included in `render_tree` message
    Scroll {
        scroll: ScrollRequest<'a>,
    },
    Debug,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageFromWindow {
    Init,
    Reload,
    FileDialog,
    DirDialog,
    GoForward,
    GoBack,
    GoTop,
    History,
    Quit,
    Search { query: String, index: Option<usize>, matcher: SearchMatcher },
    OpenFile { path: String },
    ZoomIn,
    ZoomOut,
    DragWindow,
    ToggleMaximized,
    ToggleMinimized,
    NewWindow { path: Option<String> },
    DuplicateWindow { heading: Option<usize> },
    OpenMenu { position: Option<(f64, f64)> },
    ToggleMenuBar,
    ToggleAlwaysOnTop,
    EditConfig,
    Error { message: String },
}

#[derive(Debug)]
pub enum InitScroll {
    Fragment(String),
    Heading(usize),
    Nop,
}

#[derive(Debug)]
pub struct InitFile {
    pub path: PathBuf,
    pub scroll: InitScroll,
}

impl From<PathBuf> for InitFile {
    fn from(path: PathBuf) -> Self {
        Self { path, scroll: InitScroll::Nop }
    }
}

#[derive(Debug)]
pub enum Event<WindowId> {
    WindowMessage { message: MessageFromWindow, id: WindowId },
    FileDrop { path: PathBuf, id: WindowId },
    WatchedFilesChanged(Vec<PathBuf>),
    OpenLocalPath { file: InitFile, id: WindowId },
    OpenExternalLink(String),
    Menu(MenuItem),
    NewWindow { init_file: Option<InitFile> },
    DuplicateWindow { scroll: InitScroll, id: WindowId },
    Error(Error),
}

#[derive(Debug)]
pub enum Request<WindowId> {
    Emit(Event<WindowId>),
    CreateWindow,
}

pub enum WindowEvent<W> {
    Created(W),
    Minimized(bool),
    Focused,
    Closed,
}

impl<W> fmt::Debug for WindowEvent<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowEvent::Created(_) => write!(f, "WindowEvent::Created"),
            WindowEvent::Minimized(v) => write!(f, "WindowEvent::Minimized({v})"),
            WindowEvent::Focused => write!(f, "WindowEvent::Focused"),
            WindowEvent::Closed => write!(f, "WindowEvent::Closed"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MenuItem {
    Quit,
    Forward,
    Back,
    Top,
    Reload,
    OpenFiles,
    WatchDirs,
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
    ToggleMinimizeWindow,
    ToggleMaximizeWindow,
    NewWindow,
    DuplicateWindow,
    EditConfig,
    #[cfg(not(target_os = "macos"))]
    ToggleMenuBar,
    DeleteHistory,
}

pub trait RawMessageWriter {
    type Output;
    fn write_to(self, writer: impl io::Write) -> io::Result<Self::Output>;
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ZoomLevel(u8);

impl ZoomLevel {
    // Following the same zoom factors in Chrome
    const FACTORS: [f64; 15] =
        [0.25, 0.33, 0.50, 0.67, 0.75, 0.80, 0.90, 1.00, 1.10, 1.25, 1.50, 1.75, 2.00, 2.50, 3.00];
    const MAX: u8 = Self::FACTORS.len() as u8 - 1;
    const DEFAULT: u8 = Self::MAX / 2;

    pub fn factor(self) -> f64 {
        Self::FACTORS[self.0 as usize]
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
        Self(Self::DEFAULT)
    }
}

// Renderer-agnostic window handles type considering the case where no handles are available.
pub struct WindowHandles<'a> {
    window: Result<WindowHandle<'a>, HandleError>,
    display: Result<DisplayHandle<'a>, HandleError>,
}
impl<'a> HasWindowHandle for WindowHandles<'a> {
    fn window_handle(&self) -> Result<WindowHandle<'a>, HandleError> {
        self.window.clone()
    }
}
impl<'a> HasDisplayHandle for WindowHandles<'a> {
    fn display_handle(&self) -> Result<DisplayHandle<'a>, HandleError> {
        self.display.clone()
    }
}
impl<'a> WindowHandles<'a> {
    pub fn new<W: HasWindowHandle + HasDisplayHandle>(window: &'a W) -> Self {
        Self { window: window.window_handle(), display: window.display_handle() }
    }
    pub fn unavailable() -> Self {
        Self { window: Err(HandleError::Unavailable), display: Err(HandleError::Unavailable) }
    }
}

#[derive(Debug)]
pub enum RenderingFlow {
    Continue,
    Exit(i32),
}

/// Handle to access the renderer accross threads. It is used to send the user events to the main thread
/// from another worker thread.
pub trait RendererHandle: 'static + Send + Clone {
    type WindowId: PartialEq + Eq + Hash + Clone + Copy + Debug + Send + Sync;

    fn send(&self, event: Event<Self::WindowId>);
    fn create_window(&self);
}

/// Window is responsible for rendering a single window in the rendering context.
pub trait Window {
    type Id: PartialEq + Eq + Hash + Clone + Copy + Debug + Send + Sync;

    fn send_message(&self, message: MessageToWindow<'_>) -> Result<()>;
    fn send_message_raw<W: RawMessageWriter>(&self, writer: W) -> Result<W::Output>;
    fn set_title(&self, title: &str);
    fn state(&self) -> Option<WindowState>;
    fn show(&self);
    fn hide(&self);
    fn print(&self) -> Result<()>;
    fn zoom(&mut self, level: ZoomLevel) -> Result<()>;
    fn zoom_level(&self) -> ZoomLevel;
    fn set_always_on_top(&mut self, enabled: bool);
    fn always_on_top(&self) -> bool;
    fn drag_window(&self) -> Result<()>;
    fn is_maximized(&self) -> bool;
    fn maximize(&mut self, maximized: bool);
    fn is_minimized(&self) -> bool;
    fn minimize(&mut self, minimized: bool);
    fn appearance(&self) -> WindowAppearance;
    fn show_menu_at(&self, position: Option<(f64, f64)>);
    fn toggle_menu(&mut self) -> Result<()>;
    fn save_memory(&mut self, is_low: bool) -> Result<()>;
    fn delete_cache(&mut self) -> Result<()>;
    fn handles(&self) -> WindowHandles<'_>;
    fn id(&self) -> Self::Id;
}

/// Renderer manages the entire rendering lifecycle.
pub trait Renderer: Sized {
    type WindowId: PartialEq + Eq + Hash + Clone + Copy + Debug + Send + Sync;
    type Window: Window<Id = Self::WindowId>;
    type Handle: RendererHandle<WindowId = Self::WindowId>;

    fn new(config: Rc<Config>) -> Result<Self>;
    fn create_handle(&self) -> Self::Handle;
    /// Starts the rendering execution and runs until the process exits.
    fn start<H>(self, handler: H) -> !
    where
        H: EventHandler<Window = Self::Window, WindowId = Self::WindowId> + 'static;
}

/// Event handler which listens several rendering events.
pub trait EventHandler {
    type Window: Window;
    type WindowId: PartialEq + Eq + Hash + Clone + Copy + Debug + Send + Sync;

    fn on_event(&mut self, event: Event<Self::WindowId>) -> RenderingFlow;
    fn on_window(&mut self, id: Self::WindowId, event: WindowEvent<Self::Window>) -> RenderingFlow;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zoom_level() {
        assert_eq!(ZoomLevel::default().factor(), 1.0);
    }

    #[test]
    fn zoom_level_percent() {
        assert_eq!(ZoomLevel::default().percent(), 100);
        assert_eq!(ZoomLevel(0).percent(), 25);
        assert_eq!(ZoomLevel(ZoomLevel::MAX).percent(), 300);
    }

    #[test]
    fn zoom_in() {
        let mut z = ZoomLevel(0);
        let mut prev = z.factor();
        for _ in 0..ZoomLevel::MAX {
            z = z.zoom_in().unwrap();
            assert!(prev < z.factor());
            prev = z.factor();
        }
        assert_eq!(z.zoom_in(), None);
    }

    #[test]
    fn zoom_out() {
        let mut z = ZoomLevel(ZoomLevel::MAX);
        let mut prev = z.factor();
        for _ in 0..ZoomLevel::MAX {
            z = z.zoom_out().unwrap();
            assert!(prev > z.factor());
            prev = z.factor();
        }
        assert_eq!(z.zoom_out(), None);
    }
}
