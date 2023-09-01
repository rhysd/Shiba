use crate::renderer::{MenuItem as AppMenuItem, MenuItems};
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
    _menu_bar: MenuBar,
}

impl Menu {
    pub fn new(window: &Window) -> Result<Self> {
        #[cfg(target_os = "macos")]
        const MOD: Modifiers = Modifiers::SUPER;
        #[cfg(not(target_os = "macos"))]
        const MOD: Modifiers = Modifiers::CONTROL;

        // Note: Some native menu items are not supported by Windows. Those items are actually not inserted into menu bar.

        let file_menu = Submenu::new("&File", true);
        let cmd_o = Accelerator::new(Some(MOD), Code::KeyO);
        let cmd_shift_o = Accelerator::new(Some(MOD | Modifiers::SHIFT), Code::KeyO);
        let open_file = MenuItem::new("Open File…", true, Some(cmd_o));
        let watch_dir = MenuItem::new("Watch Directory…", true, Some(cmd_shift_o));
        let print = MenuItem::new("Print…", true, None);
        file_menu.append_items(&[
            &open_file,
            &watch_dir,
            &PredefinedMenuItem::separator(),
            &print,
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::about(Some("About Shiba"), Some(metadata())),
            &PredefinedMenuItem::separator(),
        ])?;
        #[cfg(not(target_os = "windows"))]
        file_menu.append_items(&[
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
        ])?;
        let cmd_q = Accelerator::new(Some(MOD), Code::KeyQ);
        let quit = MenuItem::new("Quit", true, Some(cmd_q));
        file_menu.append_items(&[
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &quit,
        ])?;

        let edit_menu = Submenu::new("&Edit", true);
        let cmd_f = Accelerator::new(Some(MOD), Code::KeyF);
        let cmd_g = Accelerator::new(Some(MOD), Code::KeyG);
        let cmd_shift_g = Accelerator::new(Some(MOD | Modifiers::SHIFT), Code::KeyG);
        let cmd_s = Accelerator::new(Some(MOD), Code::KeyS);
        let search = MenuItem::new("Search…", true, Some(cmd_f));
        let search_next = MenuItem::new("Search Next", true, Some(cmd_g));
        let search_prev = MenuItem::new("Search Previous", true, Some(cmd_shift_g));
        let outline = MenuItem::new("Section Outline…", true, Some(cmd_s));
        edit_menu.append_items(&[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
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
        ])?;

        let display_menu = Submenu::new("&Display", true);
        let cmd_r = Accelerator::new(Some(MOD), Code::KeyR);
        let reload = MenuItem::new("Reload", true, Some(cmd_r));
        display_menu.append_items(&[&reload, &PredefinedMenuItem::separator()])?;
        #[cfg(not(target_os = "windows"))]
        display_menu.append_items(&[
            &PredefinedMenuItem::fullscreen(None),
            &PredefinedMenuItem::separator(),
        ])?;
        let cmd_plus = Accelerator::new(Some(MOD | Modifiers::SHIFT), Code::Equal);
        let cmd_minus = Accelerator::new(Some(MOD), Code::Minus);
        let zoom_in = MenuItem::new("Zoom In", true, Some(cmd_plus));
        let zoom_out = MenuItem::new("Zoom Out", true, Some(cmd_minus));
        display_menu.append_items(&[&zoom_in, &zoom_out])?;

        let history_menu = Submenu::new("History", true);
        let cmd_left_bracket = Accelerator::new(Some(MOD), Code::BracketRight);
        let cmd_right_bracket = Accelerator::new(Some(MOD), Code::BracketLeft);
        let cmd_y = Accelerator::new(Some(MOD), Code::KeyY);
        let forward = MenuItem::new("Forward", true, Some(cmd_left_bracket));
        let back = MenuItem::new("Back", true, Some(cmd_right_bracket));
        let history = MenuItem::new("History…", true, Some(cmd_y));
        history_menu.append_items(&[
            &forward,
            &back,
            &PredefinedMenuItem::separator(),
            &history,
        ])?;

        let window_menu = Submenu::new("&Window", true);
        let always_on_top = MenuItem::new("Pin/Unpin On Top", true, None);
        window_menu.append_items(&[
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::minimize(None),
            &always_on_top,
        ])?;

        let help_menu = Submenu::new("&Help", true);
        let guide = MenuItem::new("Show Guide…", true, None);
        let open_repo = MenuItem::new("Open Repository Page", true, None);
        help_menu.append_items(&[&guide, &open_repo])?;

        let menu_bar = MenuBar::with_items(&[
            &file_menu,
            &edit_menu,
            &display_menu,
            &history_menu,
            &window_menu,
            &help_menu,
        ])?;

        #[cfg(target_os = "windows")]
        {
            menu_bar.init_for_hwnd(window.hwnd() as _)?;
        }
        #[cfg(target_os = "linux")]
        {
            menu_bar.init_for_gtk_window(window.gtk_window(), window.default_vbox())?;
        }
        #[cfg(target_os = "macos")]
        {
            menu_bar.init_for_nsapp();
            window_menu.set_as_windows_menu_for_nsapp();
            help_menu.set_as_help_menu_for_nsapp();
            let _ = window;
        }

        log::debug!("Added menubar to window");

        #[rustfmt::skip]
        let ids = HashMap::from_iter({
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
        Ok(Self { ids, receiver: MenuEvent::receiver(), _menu_bar: menu_bar })
    }
}

impl MenuItems for Menu {
    type ItemId = MenuId;

    fn receive_menu_event(&self) -> Result<Option<AppMenuItem>> {
        let Ok(event) = self.receiver.try_recv() else {
            return Ok(None);
        };
        let Some(id) = self.ids.get(&event.id).copied() else {
            anyhow::bail!("Unknown menu item id: {:?}", event);
        };
        Ok(Some(id))
    }
}
