use crate::renderer::{MenuItem as AppMenuItem, MenuItems};
use anyhow::Result;
use std::collections::HashMap;
use wry::application::accelerator::Accelerator;
use wry::application::keyboard::{KeyCode, ModifiersState};
#[cfg(not(target_os = "windows"))]
use wry::application::menu::AboutMetadata;
use wry::application::menu::{MenuBar, MenuId, MenuItem, MenuItemAttributes};

pub struct MenuIds(HashMap<MenuId, AppMenuItem>);

impl MenuIds {
    pub fn set_menu(root_menu: &mut MenuBar) -> Self {
        #[cfg(target_os = "macos")]
        const MOD: ModifiersState = ModifiersState::SUPER;
        #[cfg(not(target_os = "macos"))]
        const MOD: ModifiersState = ModifiersState::CONTROL;

        // Windows / macOS / Android / iOS: The metadata is ignored on these platforms.
        #[cfg(target_os = "linux")]
        let metadata = AboutMetadata {
            version: Some("2.0.0-alpha".into()),
            authors: Some(vec!["rhysd <lin90162@yahoo.co.jp>".into()]),
            copyright: Some("Copyright (c) 2015 rhysd".into()),
            license: Some("MIT".into()),
            website: Some("https://github.com/rhysd/Shiba".into()),
            ..Default::default()
        };
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        let metadata = AboutMetadata::default();

        // Note: Some native menu items are not supported by Windows. Those items are actually not inserted into menu bar.

        let mut file_menu = MenuBar::new();
        let cmd_o = Accelerator::new(Some(MOD), KeyCode::KeyO);
        let open_file =
            file_menu.add_item(MenuItemAttributes::new("Open File…").with_accelerators(&cmd_o));
        let cmd_shift_o = Accelerator::new(Some(MOD | ModifiersState::SHIFT), KeyCode::KeyO);
        let watch_dir = file_menu
            .add_item(MenuItemAttributes::new("Watch Directory…").with_accelerators(&cmd_shift_o));
        file_menu.add_native_item(MenuItem::Separator);
        let print = file_menu.add_item(MenuItemAttributes::new("Print…"));
        file_menu.add_native_item(MenuItem::Separator);
        #[cfg(not(target_os = "windows"))]
        {
            file_menu.add_native_item(MenuItem::About("Shiba".to_string(), metadata));
            file_menu.add_native_item(MenuItem::Separator);
            file_menu.add_native_item(MenuItem::Services);
            file_menu.add_native_item(MenuItem::Separator);
        }
        file_menu.add_native_item(MenuItem::Hide);
        file_menu.add_native_item(MenuItem::HideOthers);
        file_menu.add_native_item(MenuItem::ShowAll);
        file_menu.add_native_item(MenuItem::Separator);
        let cmd_q = Accelerator::new(Some(MOD), KeyCode::KeyQ);
        let quit = file_menu.add_item(MenuItemAttributes::new("Quit").with_accelerators(&cmd_q));
        root_menu.add_submenu("File", true, file_menu);

        let mut edit_menu = MenuBar::new();
        edit_menu.add_native_item(MenuItem::Undo);
        edit_menu.add_native_item(MenuItem::Redo);
        edit_menu.add_native_item(MenuItem::Separator);
        edit_menu.add_native_item(MenuItem::Cut);
        edit_menu.add_native_item(MenuItem::Copy);
        edit_menu.add_native_item(MenuItem::Paste);
        edit_menu.add_native_item(MenuItem::SelectAll);
        edit_menu.add_native_item(MenuItem::Separator);
        let cmd_f = Accelerator::new(Some(MOD), KeyCode::KeyF);
        let search =
            edit_menu.add_item(MenuItemAttributes::new("Search…").with_accelerators(&cmd_f));
        let cmd_g = Accelerator::new(Some(MOD), KeyCode::KeyG);
        let search_next =
            edit_menu.add_item(MenuItemAttributes::new("Search Next").with_accelerators(&cmd_g));
        let cmd_shift_g = Accelerator::new(Some(MOD | ModifiersState::SHIFT), KeyCode::KeyG);
        let search_prev = edit_menu
            .add_item(MenuItemAttributes::new("Search Previous").with_accelerators(&cmd_shift_g));
        let cmd_s = Accelerator::new(Some(MOD), KeyCode::KeyS);
        let outline = edit_menu
            .add_item(MenuItemAttributes::new("Section Outline…").with_accelerators(&cmd_s));
        root_menu.add_submenu("Edit", true, edit_menu);

        let mut display_menu = MenuBar::new();
        let cmd_r = Accelerator::new(Some(MOD), KeyCode::KeyR);
        let reload =
            display_menu.add_item(MenuItemAttributes::new("Reload").with_accelerators(&cmd_r));
        display_menu.add_native_item(MenuItem::Separator);
        #[cfg(not(target_os = "windows"))]
        {
            display_menu.add_native_item(MenuItem::EnterFullScreen);
            display_menu.add_native_item(MenuItem::Separator);
        }
        let cmd_plus = Accelerator::new(Some(MOD), KeyCode::Plus);
        let zoom_in =
            display_menu.add_item(MenuItemAttributes::new("Zoom In").with_accelerators(&cmd_plus));
        let cmd_minus = Accelerator::new(Some(MOD), KeyCode::Minus);
        let zoom_out = display_menu
            .add_item(MenuItemAttributes::new("Zoom Out").with_accelerators(&cmd_minus));
        root_menu.add_submenu("Display", true, display_menu);

        let mut history_menu = MenuBar::new();
        let cmd_left_bracket = Accelerator::new(Some(MOD), KeyCode::BracketRight);
        let forward = history_menu
            .add_item(MenuItemAttributes::new("Forward").with_accelerators(&cmd_left_bracket));
        let cmd_right_bracket = Accelerator::new(Some(MOD), KeyCode::BracketLeft);
        let back = history_menu
            .add_item(MenuItemAttributes::new("Back").with_accelerators(&cmd_right_bracket));
        history_menu.add_native_item(MenuItem::Separator);
        let cmd_y = Accelerator::new(Some(MOD), KeyCode::KeyY);
        let history =
            history_menu.add_item(MenuItemAttributes::new("History…").with_accelerators(&cmd_y));
        root_menu.add_submenu("History", true, history_menu);

        let mut window_menu = MenuBar::new();
        window_menu.add_native_item(MenuItem::Minimize);
        window_menu.add_native_item(MenuItem::Zoom);
        let toggle_always_on_top =
            window_menu.add_item(MenuItemAttributes::new("Pin/Unpin On Top"));
        root_menu.add_submenu("Window", true, window_menu);

        let mut help_menu = MenuBar::new();
        let guide = help_menu.add_item(MenuItemAttributes::new("Show Guide…"));
        let open_repo = help_menu.add_item(MenuItemAttributes::new("Open Repository Page"));
        root_menu.add_submenu("Help", true, help_menu);

        log::debug!("Added menubar to window");

        #[rustfmt::skip]
        let ids = HashMap::from_iter({
            use AppMenuItem::*;

            [
                (open_file.id(),            OpenFile),
                (watch_dir.id(),            WatchDir),
                (quit.id(),                 Quit),
                (forward.id(),              Forward),
                (back.id(),                 Back),
                (reload.id(),               Reload),
                (search.id(),               Search),
                (search_next.id(),          SearchNext),
                (search_prev.id(),          SearchPrevious),
                (outline.id(),              Outline),
                (print.id(),                Print),
                (zoom_in.id(),              ZoomIn),
                (zoom_out.id(),             ZoomOut),
                (history.id(),              History),
                (toggle_always_on_top.id(), ToggleAlwaysOnTop),
                (guide.id(),                Help),
                (open_repo.id(),            OpenRepo),
            ]
        });

        Self(ids)
    }
}

impl MenuItems for MenuIds {
    type ItemId = MenuId;

    fn item_from_id(&self, id: Self::ItemId) -> Result<AppMenuItem> {
        if let Some(item) = self.0.get(&id).copied() {
            Ok(item)
        } else {
            Err(anyhow::anyhow!("Unknown menu item id: {:?}", id))
        }
    }
}
