use crate::renderer::MenuItem as AppMenuItem;
use anyhow::Result;
use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{
    AboutMetadata, ContextMenu, LogicalPosition, Menu as MenuBar, MenuEvent, MenuEventReceiver,
    MenuId, MenuItem, Position, PredefinedMenuItem, Submenu,
};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
#[cfg(target_os = "macos")]
use wry::application::platform::macos::WindowExtMacOS as _;
#[cfg(target_os = "linux")]
use wry::application::platform::unix::WindowExtUnix as _;
#[cfg(target_os = "windows")]
use wry::application::platform::windows::WindowExtWindows as _;
use wry::application::window::{Window, WindowId};

fn metadata() -> AboutMetadata {
    let mut m = AboutMetadata {
        name: Some("Shiba".into()),
        version: Some(env!("CARGO_PKG_VERSION").into()),
        copyright: Some("Copyright (c) 2015 rhysd".into()),
        license: Some("The MIT License".into()),
        website: Some("https://github.com/rhysd/Shiba".into()),
        ..Default::default()
    };

    #[cfg(not(target_os = "darwin"))]
    {
        m.authors = Some(vec![env!("CARGO_PKG_AUTHORS").into()]);
        m.comments = Some(env!("CARGO_PKG_DESCRIPTION").into());
        m.license = Some(env!("CARGO_PKG_LICENSE").into());
        m.website = Some(env!("CARGO_PKG_HOMEPAGE").into());
    }

    #[cfg(not(windows))]
    {
        use muda::Icon;
        const ICON_RGBA: &[u8] = include_bytes!("../assets/icon_256x256.rgba");
        m.icon = Some(Icon::from_rgba(ICON_RGBA.into(), 256, 256).unwrap());
    }

    m
}

pub struct MenuEvents {
    ids: HashMap<MenuId, AppMenuItem>,
    receiver: &'static MenuEventReceiver,
}

impl MenuEvents {
    pub fn new() -> Self {
        Self { ids: HashMap::new(), receiver: MenuEvent::receiver() }
    }

    pub fn try_receive(&self) -> Result<Option<AppMenuItem>> {
        let Ok(event) = self.receiver.try_recv() else {
            return Ok(None);
        };
        let Some(id) = self.ids.get(&event.id).copied() else {
            anyhow::bail!("Unknown menu item ID in event {:?}: {:?}", event, self.ids);
        };
        Ok(Some(id))
    }
}

#[derive(Clone)]
pub struct Menu {
    // This instance must be kept since dropping this instance removes menu from application
    menu_bar: MenuBar,
    visibility: HashMap<WindowId, bool>,
    #[cfg(target_os = "macos")]
    window_menu: Submenu,
    #[cfg(target_os = "macos")]
    help_menu: Submenu,
}

impl Menu {
    pub fn new(events: &mut MenuEvents) -> Result<Self> {
        fn accel(text: &str, m: Modifiers, c: Code) -> MenuItem {
            MenuItem::new(text, true, Some(Accelerator::new(Some(m), c)))
        }
        fn no_accel(text: &str) -> MenuItem {
            MenuItem::new(text, true, None)
        }

        #[cfg(target_os = "macos")]
        const MOD: Modifiers = Modifiers::SUPER;
        #[cfg(not(target_os = "macos"))]
        const MOD: Modifiers = Modifiers::CONTROL;

        let menu_bar = MenuBar::new();

        // Custom menu items
        let settings = no_accel("Settings…");
        let quit = accel("Quit", MOD, Code::KeyQ);
        let open_file = accel("Open File…", MOD, Code::KeyO);
        let watch_dir = accel("Watch Directory…", MOD | Modifiers::SHIFT, Code::KeyO);
        let print = no_accel("Print…");
        let search = accel("Search…", MOD, Code::KeyF);
        let search_next = accel("Search Next", MOD, Code::KeyG);
        let search_prev = accel("Search Previous", MOD | Modifiers::SHIFT, Code::KeyG);
        let outline = accel("Section Outline…", MOD, Code::KeyS);
        let reload = accel("Reload", MOD, Code::KeyR);
        let zoom_in = accel("Zoom In", MOD | Modifiers::SHIFT, Code::Equal); // XXX: US keyboard only
        let zoom_out = accel("Zoom Out", MOD, Code::Minus);
        #[cfg(not(target_os = "macos"))]
        let toggle_menu_bar = no_accel("Toggle Menu Bar");
        let forward = accel("Forward", MOD, Code::BracketRight);
        let back = accel("Back", MOD, Code::BracketLeft);
        let history = accel("History…", MOD, Code::KeyY);
        let always_on_top = no_accel("Pin/Unpin On Top");
        let guide = no_accel("Show Guide…");
        let open_repo = no_accel("Open Repository Page");

        // Menu bar structure
        let window_menu = Submenu::with_items(
            "&Window",
            true,
            &[
                #[cfg(not(target_os = "linux"))]
                &PredefinedMenuItem::minimize(None),
                #[cfg(target_os = "windows")]
                &PredefinedMenuItem::maximize(None),
                #[cfg(target_os = "macos")]
                &PredefinedMenuItem::fullscreen(None),
                &always_on_top,
                &PredefinedMenuItem::separator(),
                &zoom_in,
                &zoom_out,
                #[cfg(not(target_os = "macos"))]
                &PredefinedMenuItem::separator(),
                #[cfg(not(target_os = "macos"))]
                &toggle_menu_bar,
            ],
        )?;
        let help_menu = Submenu::with_items("&Help", true, &[&guide, &open_repo])?;
        menu_bar.append_items(&[
            #[cfg(target_os = "macos")]
            &Submenu::with_items(
                "Shiba",
                true,
                &[
                    &PredefinedMenuItem::about(Some("About Shiba"), Some(metadata())),
                    &PredefinedMenuItem::separator(),
                    &settings,
                    &PredefinedMenuItem::separator(),
                    &PredefinedMenuItem::services(None),
                    &PredefinedMenuItem::separator(),
                    &PredefinedMenuItem::hide(None),
                    &PredefinedMenuItem::hide_others(None),
                    &PredefinedMenuItem::show_all(None),
                    &PredefinedMenuItem::separator(),
                    &quit,
                ],
            )?,
            &Submenu::with_items(
                "&File",
                true,
                &[
                    &open_file,
                    &watch_dir,
                    &reload,
                    &PredefinedMenuItem::separator(),
                    &print,
                    #[cfg(not(target_os = "macos"))]
                    &PredefinedMenuItem::separator(),
                    #[cfg(not(target_os = "macos"))]
                    &PredefinedMenuItem::about(Some("About Shiba"), Some(metadata())),
                    #[cfg(target_os = "windows")]
                    &PredefinedMenuItem::separator(),
                    #[cfg(target_os = "windows")]
                    &PredefinedMenuItem::hide(None),
                    #[cfg(target_os = "windows")]
                    &PredefinedMenuItem::hide_others(None),
                    #[cfg(not(target_os = "macos"))]
                    &PredefinedMenuItem::separator(),
                    #[cfg(not(target_os = "macos"))]
                    &settings,
                    #[cfg(not(target_os = "macos"))]
                    &PredefinedMenuItem::separator(),
                    #[cfg(not(target_os = "macos"))]
                    &quit,
                ],
            )?,
            &Submenu::with_items(
                "&Edit",
                true,
                &[
                    #[cfg(target_os = "macos")]
                    &PredefinedMenuItem::undo(None),
                    #[cfg(target_os = "macos")]
                    &PredefinedMenuItem::redo(None),
                    #[cfg(target_os = "macos")]
                    &PredefinedMenuItem::separator(),
                    &PredefinedMenuItem::cut(None),
                    &PredefinedMenuItem::copy(None),
                    &PredefinedMenuItem::paste(None),
                    &PredefinedMenuItem::select_all(None),
                    &PredefinedMenuItem::separator(),
                    &search,
                    &search_next,
                    &search_prev,
                    &outline,
                ],
            )?,
            &Submenu::with_items(
                "History",
                true,
                &[&forward, &back, &PredefinedMenuItem::separator(), &history],
            )?,
            &window_menu,
            &help_menu,
        ])?;

        #[rustfmt::skip]
        events.ids.extend({
            use AppMenuItem::*;
            [
                (open_file.into_id(),     OpenFile),
                (watch_dir.into_id(),     WatchDir),
                (quit.into_id(),          Quit),
                (forward.into_id(),       Forward),
                (back.into_id(),          Back),
                (reload.into_id(),        Reload),
                (search.into_id(),        Search),
                (search_next.into_id(),   SearchNext),
                (search_prev.into_id(),   SearchPrevious),
                (outline.into_id(),       Outline),
                (print.into_id(),         Print),
                (zoom_in.into_id(),       ZoomIn),
                (zoom_out.into_id(),      ZoomOut),
                (history.into_id(),       History),
                (always_on_top.into_id(), ToggleAlwaysOnTop),
                (guide.into_id(),         Help),
                (open_repo.into_id(),     OpenRepo),
                (settings.into_id(),      EditConfig),
                #[cfg(not(target_os = "macos"))]
                (toggle_menu_bar.into_id(),   ToggleMenuBar),
            ]
        });

        log::debug!("Registered menu items: {:?}", events.ids);
        Ok(Self {
            menu_bar,
            visibility: HashMap::new(),
            #[cfg(target_os = "macos")]
            window_menu,
            #[cfg(target_os = "macos")]
            help_menu,
        })
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn menu_bar(&self) -> &MenuBar {
        &self.menu_bar
    }

    pub fn toggle(&mut self, window: &Window) -> Result<()> {
        let id = window.id();
        match self.visibility.entry(id) {
            Entry::Vacant(entry) => {
                #[cfg(target_os = "windows")]
                self.menu_bar.init_for_hwnd(window.hwnd() as _)?;
                #[cfg(target_os = "linux")]
                self.menu_bar.init_for_gtk_window(window.gtk_window(), window.default_vbox())?;
                #[cfg(target_os = "macos")]
                {
                    self.menu_bar.init_for_nsapp();
                    self.window_menu.set_as_windows_menu_for_nsapp();
                    self.help_menu.set_as_help_menu_for_nsapp();
                }
                entry.insert(true);
                log::debug!("Initialized menubar for window (id={:?})", id);
                Ok(())
            }
            #[cfg(target_os = "macos")]
            Entry::Occupied(_) => Ok(()), // On macOS, menu bar is always visible
            #[cfg(not(target_os = "macos"))]
            Entry::Occupied(entry) => {
                let visible = entry.into_mut();
                if *visible {
                    #[cfg(target_os = "windows")]
                    self.menu_bar.hide_for_hwnd(window.hwnd() as _)?;
                    #[cfg(target_os = "linux")]
                    self.menu_bar.hide_for_gtk_window(window.gtk_window())?;
                    log::debug!("Hide menu on window (id={:?})", id);
                } else {
                    #[cfg(target_os = "windows")]
                    self.menu_bar.show_for_hwnd(window.hwnd() as _)?;
                    #[cfg(target_os = "linux")]
                    self.menu_bar.show_for_gtk_window(window.gtk_window())?;
                    log::debug!("Show menu on window (id={:?})", id);
                }
                *visible = !*visible;
                Ok(())
            }
        }
    }

    pub fn show_at(&self, position: Option<(f64, f64)>, window: &Window) {
        let position = position.map(|(x, y)| Position::Logical(LogicalPosition { x, y }));
        log::debug!("Showing context menu at {:?}", position);
        #[cfg(target_os = "windows")]
        self.menu_bar.show_context_menu_for_hwnd(window.hwnd() as _, position);
        #[cfg(target_os = "linux")]
        self.menu_bar.show_context_menu_for_gtk_window(window.gtk_window(), position);
        #[cfg(target_os = "macos")]
        self.menu_bar.show_context_menu_for_nsview(window.ns_view() as _, position);
    }
}
