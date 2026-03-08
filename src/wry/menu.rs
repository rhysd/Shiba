use crate::renderer::{Event, EventSender, MenuItem as AppMenuItem};
use anyhow::Result;
use muda::dpi::{LogicalPosition, Position};
use muda::{
    AboutMetadata, ContextMenu, Menu as MenuBar, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};
use std::collections::HashMap;
#[cfg(target_os = "macos")]
use tao::platform::macos::WindowExtMacOS as _;
#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix as _;
#[cfg(target_os = "windows")]
use tao::platform::windows::WindowExtWindows as _;
use tao::window::Window;

fn metadata() -> AboutMetadata {
    let mut m = AboutMetadata {
        name: Some("Shiba".into()),
        version: Some(env!("CARGO_PKG_VERSION").into()),
        copyright: Some("Copyright (c) 2015 rhysd".into()),
        license: Some("The MIT License".into()),
        ..Default::default()
    };

    #[cfg(not(target_os = "macos"))]
    {
        m.authors = Some(vec![env!("CARGO_PKG_AUTHORS").into()]); // This implementation is only correct when only one person is listed in `authors` array.
        m.comments = Some(env!("CARGO_PKG_DESCRIPTION").into());
        m.license = Some(env!("CARGO_PKG_LICENSE").into());
        m.website = Some(env!("CARGO_PKG_HOMEPAGE").into());
    }

    #[cfg(not(target_os = "windows"))]
    {
        use muda::Icon;
        const ICON_RGBA: &[u8] = include_bytes!("../assets/icon_256x256.rgba");
        m.icon = Some(Icon::from_rgba(ICON_RGBA.into(), 256, 256).unwrap());
    }

    m
}

#[derive(Default, Clone)]
pub struct Menu {
    menu_bar: MenuBar, // Note: This will remove menu from application on being dropped
    #[cfg(not(target_os = "macos"))]
    visibility: Option<bool>,
}

impl Menu {
    pub fn create<S: EventSender + Sync>(&self, sender: S) -> Result<()> {
        fn item(text: &str) -> MenuItem {
            MenuItem::new(text, true, None)
        }

        // Custom menu items
        let settings = item("Settings…");
        let quit = item("Quit");
        let open_files = item("Open Files…");
        let watch_dirs = item("Watch Directories…");
        let print = item("Print…");
        let search = item("Search…");
        let search_next = item("Search Next");
        let search_prev = item("Search Previous");
        let outline = item("Section Outline…");
        let reload = item("Reload");
        let zoom_in = item("Zoom In");
        let zoom_out = item("Zoom Out");
        #[cfg(not(target_os = "macos"))]
        let toggle_menu_bar = item("Toggle Menu Bar");
        let delete_history = item("Delete History");
        let forward = item("Forward");
        let back = item("Back");
        let top = item("Latest");
        let history = item("History…");
        let minimize = item("Minimize");
        let maximize = item("Maximize");
        let always_on_top = item("Pin/Unpin On Top");
        let guide = item("Show Key Guide…");
        let open_repo = item("Open Repository Page");
        let about = PredefinedMenuItem::about(Some("About Shiba"), Some(metadata()));

        // Menu bar structure
        let window_menu = Submenu::with_items(
            "&Window",
            true,
            &[
                &minimize,
                &maximize,
                #[cfg(target_os = "macos")]
                &PredefinedMenuItem::fullscreen(None),
                &always_on_top,
                &PredefinedMenuItem::separator(),
                &zoom_in,
                &zoom_out,
                &PredefinedMenuItem::separator(),
                #[cfg(not(target_os = "macos"))]
                &toggle_menu_bar,
                #[cfg(not(target_os = "macos"))]
                &PredefinedMenuItem::bring_all_to_front(None),
            ],
        )?;
        let help_menu = Submenu::with_items("&Help", true, &[&guide, &open_repo])?;
        self.menu_bar.append_items(&[
            #[cfg(target_os = "macos")]
            &Submenu::with_items(
                "Shiba",
                true,
                &[
                    &about,
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
                    &open_files,
                    &watch_dirs,
                    &reload,
                    &PredefinedMenuItem::separator(),
                    &print,
                    #[cfg(not(target_os = "macos"))]
                    &PredefinedMenuItem::separator(),
                    #[cfg(not(target_os = "macos"))]
                    &about,
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
                &[
                    &forward,
                    &back,
                    &top,
                    &PredefinedMenuItem::separator(),
                    &history,
                    &delete_history,
                ],
            )?,
            &window_menu,
            &help_menu,
        ])?;

        #[rustfmt::skip]
        let ids = {
            use AppMenuItem::*;
            HashMap::from([
                (open_files.into_id(),      OpenFiles),
                (watch_dirs.into_id(),      WatchDirs),
                (quit.into_id(),            Quit),
                (forward.into_id(),         Forward),
                (back.into_id(),            Back),
                (top.into_id(),             Top),
                (reload.into_id(),          Reload),
                (search.into_id(),          Search),
                (search_next.into_id(),     SearchNext),
                (search_prev.into_id(),     SearchPrevious),
                (outline.into_id(),         Outline),
                (print.into_id(),           Print),
                (zoom_in.into_id(),         ZoomIn),
                (zoom_out.into_id(),        ZoomOut),
                (history.into_id(),         History),
                (always_on_top.into_id(),   ToggleAlwaysOnTop),
                (minimize.into_id(),        ToggleMinimizeWindow),
                (maximize.into_id(),        ToggleMaximizeWindow),
                (guide.into_id(),           Help),
                (open_repo.into_id(),       OpenRepo),
                (settings.into_id(),        EditConfig),
                #[cfg(not(target_os = "macos"))]
                (toggle_menu_bar.into_id(), ToggleMenuBar),
                (delete_history.into_id(),  DeleteHistory),
            ])
        };
        log::debug!("Registered menu items: {:?}", ids);

        let num_ids = ids.len();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let event = if let Some(item) = ids.get(&event.id).copied() {
                Event::Menu(item)
            } else {
                let err = anyhow::anyhow!("Unknown menu item ID in event {:?}: {:?}", event, ids);
                Event::Error(err)
            };
            sender.send(event);
        }));
        log::debug!("Set menu event handler with {} menu items", num_ids);

        // Menu bar on macOS is always visible
        #[cfg(target_os = "macos")]
        {
            self.menu_bar.init_for_nsapp();
            window_menu.set_as_windows_menu_for_nsapp();
            help_menu.set_as_help_menu_for_nsapp();
            log::debug!("Initialized menubar for macOS");
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn menu_bar(&self) -> &MenuBar {
        &self.menu_bar
    }

    #[cfg(target_os = "macos")]
    pub fn is_visible(&self) -> bool {
        true
    }
    #[cfg(not(target_os = "macos"))]
    pub fn is_visible(&self) -> bool {
        self.visibility.unwrap_or(false)
    }

    #[cfg(not(target_os = "macos"))]
    pub fn toggle(&mut self, window: &Window) -> Result<()> {
        let is_visible = match self.visibility {
            None => {
                // Safety: Using the handle returned from `Window::hwnd`.
                #[cfg(target_os = "windows")]
                unsafe {
                    self.menu_bar.init_for_hwnd(window.hwnd() as _)?;
                }
                #[cfg(target_os = "linux")]
                self.menu_bar.init_for_gtk_window(window.gtk_window(), window.default_vbox())?;
                log::debug!("Initialized menubar for window: {:?}", window.id());
                true
            }
            Some(true) => {
                // Safety: The handle is valid because it is returned from `Window::hwnd`.
                #[cfg(target_os = "windows")]
                unsafe {
                    self.menu_bar.hide_for_hwnd(window.hwnd() as _)?;
                }
                #[cfg(target_os = "linux")]
                self.menu_bar.hide_for_gtk_window(window.gtk_window())?;
                log::debug!("Hide menu on window (id={:?})", window.id());
                false
            }
            Some(false) => {
                // Safety: The handle is valid because it is returned from `Window::hwnd`.
                #[cfg(target_os = "windows")]
                unsafe {
                    self.menu_bar.show_for_hwnd(window.hwnd() as _)?;
                }
                #[cfg(target_os = "linux")]
                self.menu_bar.show_for_gtk_window(window.gtk_window())?;
                log::debug!("Show menu on window (id={:?})", window.id());
                true
            }
        };
        self.visibility = Some(is_visible);
        Ok(())
    }
    #[cfg(target_os = "macos")]
    pub fn toggle(&mut self, _window: &Window) -> Result<()> {
        Ok(()) // Menu bar on macOS is always visible
    }

    pub fn show_at(&self, position: Option<(f64, f64)>, window: &Window) {
        let position = position.map(|(x, y)| Position::Logical(LogicalPosition { x, y }));
        log::debug!("Showing context menu at {:?}", position);
        // Safety: Using the handle returned from `Window::hwnd`.
        #[cfg(target_os = "windows")]
        unsafe {
            self.menu_bar.show_context_menu_for_hwnd(window.hwnd() as _, position);
        }
        #[cfg(target_os = "linux")]
        self.menu_bar.show_context_menu_for_gtk_window(window.gtk_window().as_ref(), position);
        // Safety: Using the pointer returned from `Window::ns_view`.
        #[cfg(target_os = "macos")]
        unsafe {
            self.menu_bar.show_context_menu_for_nsview(window.ns_view() as _, position);
        }
    }
}
