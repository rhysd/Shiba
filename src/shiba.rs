use crate::cli::Options;
use crate::config::Config;
use crate::dialog::Dialog;
use crate::history::{Direction, History};
use crate::opener::Opener;
use crate::preview::Preview;
use crate::renderer::{
    Event, EventHandler, MenuItem, MessageFromWindow, MessageToWindow, Renderer, RendererHandle,
    RenderingFlow, Window, WindowHandles,
};
#[cfg(feature = "__sanity")]
use crate::sanity::SanityTest;
use crate::watcher::{PathFilter, Watcher};
use anyhow::{Context as _, Error, Result};
use std::collections::HashMap;
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;

struct WindowManager<R: Renderer> {
    windows: HashMap<R::WindowId, R::Window>,
    focused: Option<R::WindowId>,
}

impl<R: Renderer> Default for WindowManager<R> {
    fn default() -> Self {
        Self { windows: HashMap::new(), focused: None }
    }
}

impl<R: Renderer> WindowManager<R> {
    fn focused(&self) -> &R::Window {
        if let Some(id) = self.focused.as_ref() {
            self.windows.get(id).expect("Focused window must exist")
        } else {
            self.windows.values().next().expect("At least one window exist")
        }
    }

    fn focused_mut(&mut self) -> &mut R::Window {
        if let Some(id) = self.focused.as_ref() {
            self.windows.get_mut(id).expect("Focused window must exist")
        } else {
            self.windows.values_mut().next().expect("At least one window exist")
        }
    }

    fn get(&self, id: R::WindowId) -> &R::Window {
        self.windows.get(&id).unwrap_or_else(|| self.focused())
    }

    fn get_mut(&mut self, id: R::WindowId) -> &mut R::Window {
        if let Some(window) = self.windows.get_mut(&id).map(|v| v as *mut _) {
            // Safety:
            // `window` originates from `self.windows.get_mut(&key)`, so it points to a valid element
            // inside `self.windows`. Converting it to a raw pointer ends the borrow from `get_mut`.
            // In this branch we immediately reconstruct `&mut V` and return it without mutating the
            // map or creating any other mutable references to the same element. The `else` branch
            // (which calls `focused_mut`) is not executed in this case, so no aliasing mutable
            // reference can occur.
            unsafe { &mut *window }
        } else {
            log::debug!("Window ID {id:?} does not exist. Fall back to focused window");
            self.focused_mut()
        }
    }

    fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    fn add(&mut self, window: R::Window) {
        let id = window.id();
        log::debug!("Add new window: {id:?}");
        self.windows.insert(id.clone(), window);
        self.set_focus(id);
    }

    fn delete(&mut self, id: R::WindowId) -> Option<R::Window> {
        let removed = self.windows.remove(&id)?;
        log::debug!("Deleted window {id:?}");
        if self.focused == Some(id) {
            self.focused = self
                .windows
                .iter()
                .find_map(|(id, window)| window.is_focused().then(|| id.clone()));
        }
        Some(removed)
    }

    fn set_focus(&mut self, id: R::WindowId) {
        let focused = Some(id.clone());
        // Invariant: Focused window ID must exist in `self.windows`
        if self.focused != focused && self.windows.contains_key(&id) {
            log::debug!("Set focus on window: {id:?}");
            self.focused = focused;
        }
    }
}

pub struct Shiba<R: Renderer, O, W, D> {
    windows: WindowManager<R>,
    opener: O,
    history: History,
    watcher: W,
    dialog: D,
    config: Rc<Config>,
    preview: Preview,
    init_file: Option<PathBuf>,
    #[cfg(feature = "__sanity")]
    sanity: SanityTest<R::Handle>,
}

impl<R, O, W, D> Shiba<R, O, W, D>
where
    R: Renderer,
    O: Opener,
    W: Watcher,
    D: Dialog,
{
    pub fn run(mut options: Options) -> Result<()>
    where
        Self: 'static,
    {
        fn on_err<D: Dialog>(err: Error) -> Error {
            let err = err.context("Could not launch application");
            if let Ok(dialog) = D::new(&Config::default()) {
                dialog.alert(&err, &WindowHandles::unsupported());
            }
            err
        }

        log::debug!("Application options: {:?}", options);
        let watch_paths = mem::take(&mut options.watch_paths);
        let init_file = mem::take(&mut options.init_file);

        let config = Rc::new(Config::load(options)?);
        log::debug!("Application config: {:?}", config);

        let renderer = R::new(config.clone()).map_err(on_err::<D>)?;
        let dog = Self::new(watch_paths, init_file, config, &renderer).map_err(on_err::<D>)?;

        renderer.create_handle().create_window();
        renderer.start(dog)
    }

    pub fn new(
        watch_paths: Vec<PathBuf>,
        init_file: Option<PathBuf>,
        config: Rc<Config>,
        renderer: &R,
    ) -> Result<Self> {
        let filter = PathFilter::new(config.watch());
        let mut watcher = W::new(renderer.create_handle(), filter)?;
        let mut history = History::load(&config);
        for path in watch_paths {
            log::debug!("Watching initial path: {:?}", path);
            watcher.watch(&path)?;
            history.push(path);
        }

        Ok(Self {
            windows: WindowManager::default(),
            opener: O::default(),
            history,
            watcher,
            dialog: D::new(&config)?,
            config,
            preview: Preview::default(),
            init_file,
            #[cfg(feature = "__sanity")]
            sanity: SanityTest::new(renderer.create_handle()),
        })
    }

    fn open_preview(&mut self, path: PathBuf) -> Result<()> {
        self.watcher.watch(&path)?; // Watch path at first since the file may not exist yet
        if self.preview.show(&path, self.windows.focused())? {
            self.history.push(path);
        }
        Ok(())
    }

    fn navigate(&mut self, dir: Direction) -> Result<()> {
        let mut current = if self.preview.is_empty() {
            // When the welcome page is displayed, the history already indicates the latest history item.
            match dir {
                Direction::Forward => None,
                Direction::Back | Direction::Top => self.history.current(),
            }
        } else {
            self.history.navigate(dir)
        };

        while let Some(path) = current {
            log::debug!("Try to navigate preview page with direction {dir:?}: {path:?}");
            if self.preview.show(path, self.windows.focused())? {
                return Ok(());
            }
            current = self.history.delete(dir);
        }

        log::debug!("No page found in history with direction {dir:?}");
        Ok(())
    }

    fn reload(&mut self) -> Result<()> {
        if self.preview.is_empty() {
            // When content is empty, we don't need to reload the page. This happens when 'welcome' page displays just
            // after launching the app.
            log::debug!("Skipped to reload empty content");
            return Ok(());
        }
        if let Some(path) = self.history.current() {
            log::debug!("Reload current preview page: {:?}", path);
            self.preview.show(path, self.windows.focused())?;
            self.windows.focused().send_message(MessageToWindow::Reload)?;
        }
        Ok(())
    }

    fn open_files(&mut self) -> Result<()> {
        #[cfg_attr(target_os = "windows", allow(unused_mut))]
        let mut files = self.dialog.pick_files(&self.windows.focused().handles());
        #[cfg(target_os = "windows")]
        let mut files: Vec<_> = files.into_iter().flat_map(|p| p.canonicalize().ok()).collect(); // Ensure \\? at the head of the path
        log::debug!("{} files were chosen by dialog", files.len());

        let Some(last) = files.pop() else {
            return Ok(());
        };

        for file in files {
            self.watcher.watch(&file)?;
            self.history.push(file);
        }
        log::debug!("Previewing the last file chosen by dialog: {last:?}");
        self.open_preview(last)?;

        Ok(())
    }

    fn open_dirs(&mut self) -> Result<()> {
        let dirs = self.dialog.pick_dirs(&self.windows.focused().handles());
        #[cfg(target_os = "windows")]
        let dirs: Vec<_> = dirs.into_iter().flat_map(|p| p.canonicalize().ok()).collect(); // Ensure \\? at the head of the path

        log::debug!("{} directories were chosen by dialog", dirs.len());
        for dir in dirs {
            log::debug!("Watching a directory chosen by dialog: {:?}", dir);
            self.watcher.watch(&dir)?;
        }

        Ok(())
    }

    fn zoom(&mut self, zoom_in: bool) -> Result<()> {
        let level = if zoom_in {
            self.windows.focused().zoom_level().zoom_in()
        } else {
            self.windows.focused().zoom_level().zoom_out()
        };

        let Some(level) = level else {
            return Ok(());
        };

        self.windows.focused_mut().zoom(level)?;
        let percent = level.percent();
        log::debug!("Changed zoom factor: {}%", percent);
        self.windows.focused().send_message(MessageToWindow::Zoomed { percent })?;

        Ok(())
    }

    fn toggle_always_on_top(&mut self) -> Result<()> {
        let pinned = !self.windows.focused().always_on_top();
        log::debug!("Toggle always-on-top (pinned={})", pinned);
        self.windows.focused_mut().set_always_on_top(pinned);
        self.windows.focused().send_message(MessageToWindow::AlwaysOnTop { pinned })
    }

    fn toggle_maximized(&mut self) {
        let maximized = !self.windows.focused().is_maximized();
        log::debug!("Toggle maximized window (maximized={})", maximized);
        self.windows.focused_mut().maximize(maximized);
    }

    fn toggle_minimized(&mut self) {
        let minimized = !self.windows.focused().is_minimized();
        log::debug!("Toggle minimized window (minimized={})", minimized);
        self.windows.focused_mut().minimize(minimized);
    }

    fn open_config(&mut self) -> Result<()> {
        let path = self.config.config_file()?;
        log::debug!("Opening config file via menu item: {:?}", path);
        self.opener.open(&path)
    }

    fn handle_window_message(
        &mut self,
        id: R::WindowId,
        message: MessageFromWindow,
    ) -> Result<RenderingFlow> {
        use MessageFromWindow::*;
        match message {
            Init => {
                if self.config.debug() {
                    self.windows.focused().send_message(MessageToWindow::Debug)?;
                }

                self.windows.focused().send_message(MessageToWindow::Config {
                    keymaps: self.config.keymaps(),
                    search: self.config.search(),
                    home: self.preview.home_dir(),
                    window: self.windows.focused().appearance(),
                })?;

                // Open window when the content is ready. Otherwise a white window flashes when dark theme.
                self.windows.focused().show();

                if let Some(path) = mem::take(&mut self.init_file) {
                    self.open_preview(path)?;
                } else {
                    self.windows.focused().send_message(MessageToWindow::Welcome)?;
                }

                #[cfg(feature = "__sanity")]
                self.sanity.run_test(self.windows.focused().id());
            }
            Search { query, index, matcher } => {
                self.preview.search(self.windows.focused(), &query, index, matcher)?;
            }
            GoForward => self.navigate(Direction::Forward)?,
            GoBack => self.navigate(Direction::Back)?,
            GoTop if self.preview.is_empty() => self.navigate(Direction::Back)?,
            GoTop => self.navigate(Direction::Top)?,
            History => self.history.send_paths(self.windows.focused())?,
            Reload => self.reload()?,
            FileDialog => self.open_files()?,
            DirDialog => self.open_dirs()?,
            OpenFile { path } => self.open_preview(PathBuf::from(path))?,
            ZoomIn => self.zoom(true)?,
            ZoomOut => self.zoom(false)?,
            DragWindow => self.windows.focused().drag_window()?,
            ToggleMaximized => self.toggle_maximized(),
            ToggleMinimized => self.toggle_minimized(),
            Quit => return Ok(self.quit(self.windows.get(id))),
            OpenMenu { position } => self.windows.focused().show_menu_at(position),
            ToggleMenuBar => self.windows.focused_mut().toggle_menu()?,
            ToggleAlwaysOnTop => self.toggle_always_on_top()?,
            EditConfig => self.open_config()?,
            Error { message } => anyhow::bail!("Error reported from renderer: {}", message),
        }
        Ok(RenderingFlow::Continue)
    }

    fn handle_menu_item(&mut self, item: MenuItem) -> Result<RenderingFlow> {
        use MenuItem::*;

        log::debug!("Menu item was clicked: {:?}", item);
        match item {
            Quit => return Ok(self.quit(self.windows.focused())),
            Forward => self.navigate(Direction::Forward)?,
            Back => self.navigate(Direction::Back)?,
            Top if self.preview.is_empty() => self.navigate(Direction::Back)?,
            Top => self.navigate(Direction::Top)?,
            Reload => self.reload()?,
            OpenFiles => self.open_files()?,
            WatchDirs => self.open_dirs()?,
            Search => self.windows.focused().send_message(MessageToWindow::Search)?,
            SearchNext => self.windows.focused().send_message(MessageToWindow::SearchNext)?,
            SearchPrevious => {
                self.windows.focused().send_message(MessageToWindow::SearchPrevious)?
            }
            Outline => self.windows.focused().send_message(MessageToWindow::Outline)?,
            Print if self.preview.is_empty() => {} // Do not print welcome page
            Print => self.windows.focused().print()?,
            ZoomIn => self.zoom(true)?,
            ZoomOut => self.zoom(false)?,
            #[cfg(not(target_os = "macos"))]
            ToggleMenuBar => self.window.focused_mut().toggle_menu()?,
            History => self.history.send_paths(self.windows.focused())?,
            ToggleAlwaysOnTop => self.toggle_always_on_top()?,
            ToggleMinimizeWindow => self.toggle_minimized(),
            ToggleMaximizeWindow => self.toggle_maximized(),
            Help => self.windows.focused().send_message(MessageToWindow::Help)?,
            OpenRepo => self.opener.open("https://github.com/rhysd/Shiba")?,
            EditConfig => self.open_config()?,
            DeleteHistory => {
                if self.dialog.yes_no(
                    "Deleting history...",
                    "Are you sure you want to delete all history?",
                    &self.windows.focused().handles(),
                ) {
                    self.history.clear(&self.config)?;
                    self.windows.focused_mut().delete_cache()?;
                }
            }
        }
        Ok(RenderingFlow::Continue)
    }

    fn handle_event(&mut self, event: Event<R::WindowId>) -> Result<RenderingFlow> {
        log::debug!("Handling event {:?}", event);
        match event {
            Event::WindowMessage { message, id } => return self.handle_window_message(id, message),
            Event::FileDrop { mut path, id } => {
                log::debug!("Previewing file dropped into window: {:?}", path);
                if !path.is_absolute() {
                    path = path.canonicalize()?;
                }
                self.open_preview(path)?;
            }
            Event::WatchedFilesChanged(mut paths) => {
                log::debug!("Files changed: {:?}", paths);
                if let Some(current) = self.history.current()
                    && paths.iter().any(|p| p == current)
                {
                    self.preview.show(current, self.windows.focused())?;
                    return Ok(RenderingFlow::Continue);
                }
                // Choose the last one to preview if the current file is not included in `paths`
                if let Some(mut path) = paths.pop() {
                    if !path.is_absolute() {
                        path = path.canonicalize()?;
                    }
                    if self.preview.show(&path, self.windows.focused())? {
                        self.history.push(path);
                    }
                }
            }
            Event::OpenLocalPath { mut path, id } => {
                if path.is_relative()
                    && let Some(current_file) = self.history.current()
                    && let Some(dir) = current_file.parent()
                {
                    path = dir.join(path).canonicalize()?;
                }
                let path = path;
                let is_markdown = self.config.watch().file_extensions.matches(&path);
                if is_markdown {
                    log::debug!("Opening local markdown link clicked in WebView: {:?}", path);
                    self.open_preview(path)?;
                } else {
                    log::debug!("Opening local link item clicked in WebView: {:?}", path);
                    self.opener.open(&path).with_context(|| format!("opening path {:?}", &path))?;
                }
            }
            Event::OpenExternalLink(link) => {
                log::debug!("Opening external link item clicked in WebView: {:?}", link);
                self.opener.open(&link).with_context(|| format!("opening link {:?}", &link))?;
            }
            Event::Menu(item) => return self.handle_menu_item(item),
            Event::Error(err) => self.dialog.alert(&err, &self.windows.focused().handles()),
        }
        Ok(RenderingFlow::Continue)
    }

    fn save(&self, last_window: &R::Window) -> Result<()> {
        log::debug!("Save the persistent data before quit with window {:?}", last_window.id());

        let mut result = Ok(()); // Don't return early
        if self.config.window().restore
            && let Some(state) = last_window.state()
            && state.height > 0.0
            && state.width > 0.0
        {
            log::debug!("Saving window state as persistent data: {:?}", state);
            result = self.config.data_dir().save(&state);
        }

        result.or(self.history.save(&self.config))
    }

    fn quit(&self, last_window: &R::Window) -> RenderingFlow {
        last_window.hide();
        if let Err(error) = self.save(last_window) {
            let error = error.context("Error while quitting the application");
            self.dialog.alert(&error, &last_window.handles());
            RenderingFlow::Exit(1)
        } else {
            RenderingFlow::Exit(0)
        }
    }
}

impl<R, O, W, D> EventHandler for Shiba<R, O, W, D>
where
    R: Renderer,
    O: Opener,
    W: Watcher,
    D: Dialog,
{
    type Window = R::Window;
    type WindowId = R::WindowId;

    fn on_event(&mut self, event: Event<Self::WindowId>) -> RenderingFlow {
        self.handle_event(event).unwrap_or_else(|err| {
            let err = err.context("Could not handle event");
            self.dialog.alert(&err, &self.windows.focused().handles());
            RenderingFlow::Continue
        })
    }

    fn on_window_created(&mut self, window: Self::Window) -> RenderingFlow {
        self.windows.add(window);
        RenderingFlow::Continue
    }

    fn on_window_minimized(&mut self, is_minimized: bool, id: Self::WindowId) -> RenderingFlow {
        if let Err(err) = self.windows.focused_mut().save_memory(is_minimized) {
            let err = err.context("Could not save memory on minimized window");
            self.dialog.alert(&err, &self.windows.focused().handles());
        }
        RenderingFlow::Continue
    }

    fn on_window_focused(&mut self, id: Self::WindowId) -> RenderingFlow {
        self.windows.set_focus(id);
        RenderingFlow::Continue
    }

    fn on_window_closed(&mut self, id: Self::WindowId) -> RenderingFlow {
        let Some(window) = self.windows.delete(id.clone()) else {
            log::error!("Window was closed but it is not managed by Shiba: {id:?}");
            return RenderingFlow::Continue;
        };

        if self.windows.is_empty() {
            log::debug!("No window remains after the last window was closed");
            self.quit(&window)
        } else {
            RenderingFlow::Continue
        }
    }
}
