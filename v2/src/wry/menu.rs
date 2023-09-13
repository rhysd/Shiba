use crate::renderer::MenuItem as AppMenuItem;
use anyhow::Result;
use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{
    AboutMetadata, Menu as MenuBar, MenuEvent, MenuEventReceiver, MenuId, MenuItem,
    PredefinedMenuItem, Submenu,
};
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use wry::application::platform::unix::WindowExtUnix as _;
#[cfg(windows)]
use wry::application::platform::windows::WindowExtWindows as _;
use wry::application::window::Window;

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

pub struct Menu {
    ids: HashMap<MenuId, AppMenuItem>,
    receiver: &'static MenuEventReceiver,
    // This instance must be kept since dropping this instance removes menu from application
    menu_bar: MenuBar,
}

impl Menu {
    pub fn new() -> Self {
        Self { ids: HashMap::new(), receiver: MenuEvent::receiver(), menu_bar: MenuBar::new() }
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn menu_bar(&self) -> &MenuBar {
        &self.menu_bar
    }

    pub fn setup(&mut self, window: &Window) -> Result<()> {
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

        // Custom menu items
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
            ],
        )?;
        let help_menu = Submenu::with_items("&Help", true, &[&guide, &open_repo])?;
        self.menu_bar.append_items(&[
            #[cfg(target_os = "macos")]
            &Submenu::with_items(
                "Shiba",
                true,
                &[
                    &PredefinedMenuItem::about(Some("About Shiba"), Some(metadata())),
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

        #[cfg(target_os = "windows")]
        {
            self.menu_bar.init_for_hwnd(window.hwnd() as _)?;
        }
        #[cfg(target_os = "linux")]
        {
            self.menu_bar.init_for_gtk_window(window.gtk_window(), window.default_vbox())?;
        }
        #[cfg(target_os = "macos")]
        {
            self.menu_bar.init_for_nsapp();
            window_menu.set_as_windows_menu_for_nsapp();
            help_menu.set_as_help_menu_for_nsapp();
            let _ = window;
        }

        log::debug!("Added menubar to window");

        #[rustfmt::skip]
        self.ids.extend({
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
            ]
        });

        log::debug!("Registered menu items: {:?}", self.ids);
        Ok(())
    }

    pub fn try_receive_event(&self) -> Result<Option<AppMenuItem>> {
        let Ok(event) = self.receiver.try_recv() else {
            return Ok(None);
        };
        let Some(id) = self.ids.get(&event.id).copied() else {
            anyhow::bail!("Unknown menu item id: {:?}", event);
        };
        Ok(Some(id))
    }
}
