use crate::cli::Options;
use crate::config::{Config, home_dir};
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
use anyhow::{Context as _, Result};
use std::collections::{HashMap, VecDeque};
use std::mem;
use std::path::{Path, PathBuf};
use std::rc::Rc;

struct WindowManager<R: Renderer> {
    windows: HashMap<R::WindowId, (R::Window, Preview)>,
    focused: Option<R::WindowId>,
}

impl<R: Renderer> Default for WindowManager<R> {
    fn default() -> Self {
        Self { windows: HashMap::new(), focused: None }
    }
}

impl<R: Renderer> WindowManager<R> {
    pub fn focused(&self) -> (&R::Window, &Preview) {
        let (win, prev) = if let Some(id) = &self.focused {
            self.windows.get(id).expect("Focused window must exist")
        } else {
            self.windows.values().next().expect("At least one window exist")
        };
        (win, prev)
    }

    pub fn focused_mut(&mut self) -> (&mut R::Window, &mut Preview) {
        let (win, prev) = if let Some(id) = self.focused.as_ref() {
            self.windows.get_mut(id).expect("Focused window must exist")
        } else {
            self.windows.values_mut().next().expect("At least one window exist")
        };
        (win, prev)
    }

    pub fn get(&self, id: R::WindowId) -> (&R::Window, &Preview) {
        if let Some((win, prev)) = self.windows.get(&id) { (win, prev) } else { self.focused() }
    }

    pub fn get_mut(&mut self, id: R::WindowId) -> (&mut R::Window, &mut Preview) {
        if let Some(ptr) = self.windows.get_mut(&id).map(|v| v as *mut _) {
            // Safety:
            // `ptr` originates from `self.windows.get_mut(&key)`, so it points to a valid element
            // inside `self.windows`. Converting it to a raw pointer ends the borrow from `get_mut`.
            // In this branch we immediately reconstruct `&mut V` and return it without mutating the
            // map or creating any other mutable references to the same element. The `else` branch
            // (which calls `focused_mut`) is not executed in this case, so no aliasing mutable
            // reference can occur.
            let (win, prev) = unsafe { &mut *ptr };
            (win, prev)
        } else {
            log::debug!("Window ID {id:?} does not exist. Fall back to focused window");
            self.focused_mut()
        }
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (R::WindowId, &mut R::Window, &mut Preview)> {
        self.windows.iter_mut().map(|(&i, (w, p))| (i, w, p))
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn add(&mut self, window: R::Window) {
        let id = window.id();
        log::debug!("Add new window: {id:?}");
        self.windows.insert(id, (window, Preview::default()));
        self.set_focus(id);
    }

    pub fn delete(&mut self, id: R::WindowId) -> Option<(R::Window, Preview)> {
        let removed = self.windows.remove(&id)?;
        log::debug!("Deleted window {id:?}");
        if self.focused == Some(id) {
            // XXX: Should we set `None` here or fall back to the first window to avoid `None`?
            self.focused = self
                .windows
                .iter()
                .find_map(|(id, (window, _))| window.is_focused().then_some(*id));
            log::debug!("Updated focus to window {:?}", self.focused);
        }
        Some(removed)
    }

    pub fn set_focus(&mut self, id: R::WindowId) {
        // Invariant: Focused window ID must exist in `self.windows`
        assert!(self.windows.contains_key(&id));
        log::debug!("Set focused window: {id:?}");
        self.focused = Some(id);
    }

    pub fn focused_id(&self) -> R::WindowId {
        self.focused
            .unwrap_or_else(|| *self.windows.keys().next().expect("At least one window exists"))
    }
}

pub struct Shiba<R: Renderer, O, W, D> {
    renderer: R::Handle,
    windows: WindowManager<R>,
    opener: O,
    history: History,
    watcher: W,
    dialog: D,
    config: Rc<Config>,
    preview: Preview,
    init_files: VecDeque<PathBuf>,
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
    pub fn run(options: Options) -> Result<()>
    where
        Self: 'static,
    {
        let Err(error) = Self::run_impl(options) else {
            return Ok(());
        };

        let error = error.context("Could not launch application");
        if let Ok(dialog) = D::new(&Config::default()) {
            dialog.alert(&error, &WindowHandles::unsupported());
        }
        Err(error)
    }

    fn run_impl(mut options: Options) -> Result<()>
    where
        Self: 'static,
    {
        log::debug!("Application options: {:?}", options);
        let watch_paths = mem::take(&mut options.watch_paths);
        let init_file = mem::take(&mut options.init_file);

        let config = Rc::new(Config::load(options)?);
        log::debug!("Application config: {:?}", config);

        let renderer = R::new(config.clone())?;
        let dog = Self::new(watch_paths, init_file, config, &renderer)?;

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
        let handle = renderer.create_handle();
        handle.create_window();

        Ok(Self {
            renderer: handle,
            windows: WindowManager::default(),
            opener: O::default(),
            history,
            watcher,
            dialog: D::new(&config)?,
            config,
            preview: Preview::default(),
            init_files: init_file.into_iter().collect(),
            #[cfg(feature = "__sanity")]
            sanity: SanityTest::new(renderer.create_handle()),
        })
    }

    fn open_preview(&mut self, id: R::WindowId, path: PathBuf) -> Result<()> {
        self.watcher.watch(&path)?; // Watch path at first since the file may not exist yet
        let (window, preview) = self.windows.get_mut(id);
        if preview.show(&path, window)? {
            self.history.push(path);
        }
        Ok(())
    }

    fn navigate(&mut self, id: R::WindowId, dir: Direction) -> Result<()> {
        let (window, preview) = self.windows.get_mut(id);

        let (mut current, dir) = if preview.is_empty() {
            // When the welcome page is displayed, the history already indicates the latest history item.
            match dir {
                Direction::Forward => (None, dir),
                Direction::Back | Direction::Top => (self.history.current(), Direction::Back),
            }
        } else {
            (self.history.navigate(dir), dir)
        };

        while let Some(path) = current {
            log::debug!("Try to navigate preview page with direction {dir:?}: {path:?}");
            if preview.show(path, window)? {
                return Ok(());
            }
            current = self.history.delete(dir);
        }

        log::debug!("No page found in history with direction {dir:?}");
        Ok(())
    }

    fn reload(&mut self, id: R::WindowId) -> Result<()> {
        let (window, preview) = self.windows.get_mut(id);
        if preview.is_empty() {
            // When content is empty, we don't need to reload the page. This happens when 'welcome' page displays just
            // after launching the app.
            log::debug!("Skipped to reload empty content");
            return Ok(());
        }
        if let Some(path) = self.history.current() {
            log::debug!("Reload current preview page: {:?}", path);
            preview.show(path, window)?;
            window.send_message(MessageToWindow::Reload)?;
        }
        Ok(())
    }

    fn open_files(&mut self, id: R::WindowId) -> Result<()> {
        let (window, _) = self.windows.get(id);
        #[cfg_attr(target_os = "windows", allow(unused_mut))]
        let mut files = self.dialog.pick_files(&window.handles());
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
        log::debug!("Preview the last file chosen by dialog: {last:?}");
        self.open_preview(id, last)?;

        Ok(())
    }

    fn open_dirs(&mut self, id: R::WindowId) -> Result<()> {
        let (window, _) = self.windows.get(id);
        let dirs = self.dialog.pick_dirs(&window.handles());
        #[cfg(target_os = "windows")]
        let dirs: Vec<_> = dirs.into_iter().flat_map(|p| p.canonicalize().ok()).collect(); // Ensure \\? at the head of the path

        log::debug!("{} directories were chosen by dialog", dirs.len());
        for dir in dirs {
            log::debug!("Watching a directory chosen by dialog: {:?}", dir);
            self.watcher.watch(&dir)?;
        }

        Ok(())
    }

    fn zoom(&mut self, id: R::WindowId, zoom_in: bool) -> Result<()> {
        let (window, _) = self.windows.get_mut(id);
        let level = window.zoom_level();
        let level = if zoom_in { level.zoom_in() } else { level.zoom_out() };

        let Some(level) = level else {
            return Ok(());
        };

        window.zoom(level)?;
        let percent = level.percent();
        log::debug!("Changed zoom factor: {}%", percent);
        window.send_message(MessageToWindow::Zoomed { percent })?;

        Ok(())
    }

    fn toggle_always_on_top(&mut self, id: R::WindowId) -> Result<()> {
        let (window, _) = self.windows.get_mut(id);
        let pinned = !window.always_on_top();
        log::debug!("Toggle always-on-top (pinned={})", pinned);
        window.set_always_on_top(pinned);
        window.send_message(MessageToWindow::AlwaysOnTop { pinned })
    }

    fn toggle_maximized(&mut self, id: R::WindowId) {
        let (window, _) = self.windows.get_mut(id);
        let maximized = !window.is_maximized();
        log::debug!("Toggle maximized window (maximized={})", maximized);
        window.maximize(maximized);
    }

    fn toggle_minimized(&mut self, id: R::WindowId) {
        let (window, _) = self.windows.get_mut(id);
        let minimized = !window.is_minimized();
        log::debug!("Toggle minimized window (minimized={})", minimized);
        window.minimize(minimized);
    }

    fn open_config(&mut self) -> Result<()> {
        let path = self.config.config_file()?;
        log::debug!("Opening config file via menu item: {:?}", path);
        self.opener.open(&path)
    }

    fn is_markdown_file(&self, path: &Path) -> bool {
        self.config.watch().file_extensions.matches(path)
            && path.metadata().map(|md| !md.is_dir()).unwrap_or(false)
    }

    fn duplicate_window(&mut self, id: R::WindowId) {
        let (_, preview) = self.windows.get(id);
        if !preview.is_empty() {
            let path = preview.path().to_path_buf();
            log::debug!("Duplicate window with path: {path:?}");
            self.init_files.push_back(path);
        }
        self.renderer.create_window();
    }

    fn handle_window_message(
        &mut self,
        id: R::WindowId,
        message: MessageFromWindow,
    ) -> Result<RenderingFlow> {
        use MessageFromWindow::*;
        match message {
            Init => {
                let (window, _) = self.windows.get(id);

                if self.config.debug() {
                    window.send_message(MessageToWindow::Debug)?;
                }

                window.send_message(MessageToWindow::Config {
                    keymaps: self.config.keymaps(),
                    search: self.config.search(),
                    home: home_dir(),
                    window: window.appearance(),
                })?;

                // Open window when the content is ready. Otherwise a white window flashes when dark theme.
                window.show();

                if let Some(path) = self.init_files.pop_front() {
                    self.open_preview(id, path)?;
                } else {
                    window.send_message(MessageToWindow::Welcome)?;
                }

                #[cfg(feature = "__sanity")]
                self.sanity.run_test(id);
            }
            Search { query, index, matcher } => {
                let (window, preview) = self.windows.get(id);
                preview.search(window, &query, index, matcher)?;
            }
            GoForward => self.navigate(id, Direction::Forward)?,
            GoBack => self.navigate(id, Direction::Back)?,
            GoTop => self.navigate(id, Direction::Top)?,
            History => self.history.send_paths(self.windows.get(id).0)?,
            Reload => self.reload(id)?,
            FileDialog => self.open_files(id)?,
            DirDialog => self.open_dirs(id)?,
            OpenFile { path } => self.open_preview(id, PathBuf::from(path))?,
            ZoomIn => self.zoom(id, true)?,
            ZoomOut => self.zoom(id, false)?,
            DragWindow => self.windows.get(id).0.drag_window()?,
            ToggleMaximized => self.toggle_maximized(id),
            ToggleMinimized => self.toggle_minimized(id),
            NewWindow => self.renderer.create_window(),
            DuplicateWindow => self.duplicate_window(id),
            Quit => return Ok(self.quit(self.windows.get(id).0)),
            OpenMenu { position } => self.windows.get(id).0.show_menu_at(position),
            ToggleMenuBar => self.windows.get_mut(id).0.toggle_menu()?,
            ToggleAlwaysOnTop => self.toggle_always_on_top(id)?,
            EditConfig => self.open_config()?,
            Error { message } => anyhow::bail!("Error reported from renderer: {message}"),
        }
        Ok(RenderingFlow::Continue)
    }

    fn handle_menu_item(&mut self, item: MenuItem) -> Result<RenderingFlow> {
        use MenuItem::*;

        // muda doesn't provide a way to know which menu item came from which window. However, when
        // menu item is selected, the window must be focused. We can assume that the focused window
        // emitted the menu event here.
        let id = self.windows.focused_id();
        log::debug!("Menu item was clicked with window {:?}: {:?}", id, item);

        match item {
            Quit => return Ok(self.quit(self.windows.get(id).0)),
            Forward => self.navigate(id, Direction::Forward)?,
            Back => self.navigate(id, Direction::Back)?,
            Top => self.navigate(id, Direction::Top)?,
            Reload => self.reload(id)?,
            OpenFiles => self.open_files(id)?,
            WatchDirs => self.open_dirs(id)?,
            Search => self.windows.get(id).0.send_message(MessageToWindow::Search)?,
            SearchNext => self.windows.get(id).0.send_message(MessageToWindow::SearchNext)?,
            SearchPrevious => {
                self.windows.get(id).0.send_message(MessageToWindow::SearchPrevious)?
            }
            Outline => self.windows.get(id).0.send_message(MessageToWindow::Outline)?,
            Print if self.preview.is_empty() => {} // Do not print welcome page
            Print => self.windows.get(id).0.print()?,
            ZoomIn => self.zoom(id, true)?,
            ZoomOut => self.zoom(id, false)?,
            #[cfg(not(target_os = "macos"))]
            ToggleMenuBar => self.windows.get_mut(id).0.toggle_menu()?,
            History => self.history.send_paths(self.windows.get(id).0)?,
            ToggleAlwaysOnTop => self.toggle_always_on_top(id)?,
            ToggleMinimizeWindow => self.toggle_minimized(id),
            ToggleMaximizeWindow => self.toggle_maximized(id),
            NewWindow => self.renderer.create_window(),
            DuplicateWindow => self.duplicate_window(id),
            Help => self.windows.get(id).0.send_message(MessageToWindow::Help)?,
            OpenRepo => self.opener.open("https://github.com/rhysd/Shiba")?,
            EditConfig => self.open_config()?,
            DeleteHistory => {
                if self.dialog.yes_no(
                    "Deleting history...",
                    "Are you sure you want to delete all history?",
                    &self.windows.get(id).0.handles(),
                ) {
                    self.history.clear(&self.config)?;
                    self.windows.get_mut(id).0.delete_cache()?;
                }
            }
        }
        Ok(RenderingFlow::Continue)
    }

    fn handle_file_changes(&mut self, mut paths: Vec<PathBuf>) -> Result<()> {
        log::debug!("Files changed: {:?}", paths);

        let mut updated = vec![];
        let focused_id = self.windows.focused_id();
        let mut focused_window_updated = false;
        for (id, window, preview) in self.windows.iter_mut() {
            let is_updated = if let Some(idx) = paths.iter().position(|p| p == preview.path()) {
                let path = paths.swap_remove(idx);
                log::debug!("Update the preview for the file change: {:?}", path);
                preview.show(&path, window)?;
                updated.push(path);
                true
            } else if let Some(path) = updated.iter().find(|&p| p == preview.path()) {
                log::debug!("Update the (duplicate) preview for the file change: {:?}", path);
                preview.show(path, window)?;
                true
            } else {
                false
            };
            if is_updated && id == focused_id {
                focused_window_updated = true;
            }
        }

        if !focused_window_updated && let Some(path) = paths.pop() {
            log::debug!(
                "Show the new preview for the file change in window {focused_id:?}: {path:?}",
            );
            let (window, preview) = self.windows.get_mut(focused_id);
            if preview.show(&path, window)? {
                self.history.push(path);
            }
        }

        Ok(())
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
                self.open_preview(id, path)?;
            }
            Event::WatchedFilesChanged(paths) => self.handle_file_changes(paths)?,
            Event::OpenLocalPath { mut path, id } => {
                if let Some(abs_path) = self.history.absolute_path(&path) {
                    path = abs_path;
                }
                if self.is_markdown_file(&path) {
                    log::debug!("Opening local markdown link clicked in WebView: {:?}", path);
                    self.open_preview(id, path)?;
                } else {
                    log::debug!("Opening local link item clicked in WebView: {:?}", path);
                    self.opener.open(&path).with_context(|| format!("Opening path {path:?}"))?;
                }
            }
            Event::OpenExternalLink(link) => {
                log::debug!("Opening external link item clicked in WebView: {:?}", link);
                self.opener.open(&link).with_context(|| format!("opening link {:?}", &link))?;
            }
            Event::Menu(item) => return self.handle_menu_item(item),
            Event::NewWindow { init_file: None } => self.renderer.create_window(),
            Event::NewWindow { init_file: Some(mut path) } => {
                if let Some(abs_path) = self.history.absolute_path(&path) {
                    path = abs_path;
                }
                if self.is_markdown_file(&path) {
                    if let Some((id, window, _)) =
                        self.windows.iter_mut().find(|(_, _, preview)| preview.path() == path)
                    {
                        log::debug!("Path is already opened in window {:?}: {:?}", id, path);
                        window.focus();
                    } else {
                        log::debug!("Creating new window with file: {:?}", path);
                        self.init_files.push_back(path);
                        self.renderer.create_window();
                    }
                } else {
                    log::debug!("Opening local link item clicked in WebView: {:?}", path);
                    self.opener.open(&path).with_context(|| format!("opening path {:?}", &path))?;
                }
            }
            Event::Error(err) => self.dialog.alert(&err, &self.windows.focused().0.handles()),
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
            self.dialog.alert(&err, &self.windows.focused().0.handles());
            RenderingFlow::Continue
        })
    }

    fn on_window_created(&mut self, window: Self::Window) -> RenderingFlow {
        self.windows.add(window);
        RenderingFlow::Continue
    }

    fn on_window_minimized(&mut self, is_minimized: bool, _id: Self::WindowId) -> RenderingFlow {
        if let Err(err) = self.windows.focused_mut().0.save_memory(is_minimized) {
            let err = err.context("Could not save memory on minimized window");
            self.dialog.alert(&err, &self.windows.focused().0.handles());
        }
        RenderingFlow::Continue
    }

    fn on_window_focused(&mut self, id: Self::WindowId) -> RenderingFlow {
        self.windows.set_focus(id);
        RenderingFlow::Continue
    }

    fn on_window_closed(&mut self, id: Self::WindowId) -> RenderingFlow {
        let Some((window, _)) = self.windows.delete(id) else {
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
