use crate::config::{Config, KeyAction, Search as SearchConfig, SearchMatcher};
use crate::persistent::WindowState;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WindowAppearance {
    pub title: bool,
    pub vibrancy: bool,
    pub scroll_bar: bool,
    pub border_top: bool,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageToRenderer<'a> {
    Config {
        keymaps: &'a HashMap<String, KeyAction>,
        search: &'a SearchConfig,
        theme: Theme,
        recent: &'a [&'a Path],
        home: Option<&'a Path>,
        window: WindowAppearance,
    },
    Search,
    SearchNext,
    SearchPrevious,
    Welcome,
    Outline,
    PathChanged {
        path: &'a Path,
    },
    History,
    Help,
    Zoomed {
        percent: u16,
    },
    Reload,
    Debug,
    AlwaysOnTop {
        pinned: bool,
    },
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
    ZoomIn,
    ZoomOut,
    DragWindow,
    ToggleMaximized,
    OpenMenu { position: Option<(f64, f64)> },
    ToggleMenuBar,
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
    EditConfig,
    #[cfg(not(target_os = "macos"))]
    ToggleMenuBar,
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

#[derive(Debug)]
pub enum RenderingFlow {
    Continue,
    Close,
}

/// Sender to send [`UserEvent`] accross threads. It is used to send the user events to the main thread
/// from another worker thread.
pub trait UserEventSender: 'static + Send {
    fn send(&self, event: UserEvent);
}

/// Renderer is responsible for rendering the actual UI in the rendering context.
pub trait Renderer {
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
    fn drag_window(&self) -> Result<()>;
    fn is_maximized(&self) -> bool;
    fn set_maximized(&mut self, maximized: bool);
    fn window_appearance(&self) -> WindowAppearance;
    fn show_menu_at(&self, position: Option<(f64, f64)>);
    fn toggle_menu(&mut self) -> Result<()>;
}

/// Context to execute rendering.
pub trait Rendering: Sized {
    type UserEventSender: UserEventSender;
    type Renderer: Renderer;

    fn new() -> Result<Self>;
    fn create_sender(&self) -> Self::UserEventSender;
    fn create_renderer(&mut self, config: &Config) -> Result<Self::Renderer>;
    fn run<H: EventHandler>(self, handler: H) -> Result<()>;
}

/// Event handler which listens several rendering events.
pub trait EventHandler {
    fn handle_user_event(&mut self, event: UserEvent) -> Result<RenderingFlow>;
    fn handle_menu_event(&mut self, item: MenuItem) -> Result<RenderingFlow>;
    fn handle_close(&mut self) -> Result<()>;
    fn handle_error(&mut self, err: Error) -> RenderingFlow;
    fn handle_exit(&mut self) -> Result<()>;
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
