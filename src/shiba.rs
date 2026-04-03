use crate::cli::Options;
use crate::config::{Config, home_dir};
use crate::dialog::Dialog;
use crate::history::{Direction, History};
use crate::opener::Opener;
use crate::process_singleton::ProcessSingleton;
use crate::renderer::{
    Event, EventHandler, InitFile, InitScroll, MenuItem, MessageFromWindow, MessageToWindow,
    Renderer, RendererHandle, RenderingFlow, ScrollRequest, Window, WindowEvent, WindowHandles,
};
#[cfg(feature = "__sanity")]
use crate::sanity::SanityTest;
use crate::watcher::{PathFilter, Watcher};
use crate::window::WindowManager;
use anyhow::{Context as _, Error, Result};
use std::collections::VecDeque;
use std::mem;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct Shiba<R: Renderer, O, W, D> {
    renderer: R::Handle,
    windows: WindowManager<R>,
    opener: O,
    history: History,
    watcher: W,
    dialog: D,
    config: Rc<Config>,
    init_files: VecDeque<InitFile>,
    singleton: ProcessSingleton,
    exit_status: i32,
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
        Self::run_impl(options).map_err(|error| {
            let error = error.context("Could not launch application");
            if let Ok(dialog) = D::new(&Config::default()) {
                dialog.alert(&error, &WindowHandles::unavailable());
            }
            error
        })
    }

    fn run_impl(mut options: Options) -> Result<()>
    where
        Self: 'static,
    {
        log::debug!("Application options: {:?}", options);
        let paths = mem::take(&mut options.paths);

        let config = Rc::new(Config::load(options)?);
        log::debug!("Application config: {:?}", config);

        let singleton = if config.process().singleton {
            #[cfg(not(target_os = "windows"))]
            {
                ProcessSingleton::with_socket_file(config.data_dir())
            }
            #[cfg(target_os = "windows")]
            {
                ProcessSingleton::with_default_namespace()
            }
        } else {
            log::debug!("Disable process singleton due to user's preference");
            ProcessSingleton::default()
        };

        if singleton
            .send(&paths)
            .context("Could not connect to IPC socket for process singleton")?
        {
            return Ok(());
        }

        let watch_paths = paths.watched;
        let mut init_files = paths.additional_windows;
        if let Some(path) = paths.init {
            init_files.push(path);
        }

        let renderer = R::new(config.clone())?;
        let dog = Self::new(watch_paths, init_files, config, singleton, &renderer)?;

        renderer.start(dog)
    }

    pub fn new(
        watch_paths: Vec<PathBuf>,
        init_files: Vec<PathBuf>,
        config: Rc<Config>,
        singleton: ProcessSingleton,
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
        for _ in 0..init_files.len().max(1) {
            handle.create_window();
        }
        let init_files = init_files.into_iter().map(|p| p.into()).collect();

        Ok(Self {
            renderer: handle,
            windows: WindowManager::default(),
            opener: O::default(),
            history,
            watcher,
            dialog: D::new(&config)?,
            config,
            init_files,
            singleton,
            exit_status: 0,
        })
    }

    fn open_preview(&mut self, id: R::WindowId, file: InitFile) -> Result<&R::Window> {
        let InitFile { path, scroll } = file;
        self.watcher.watch(&path)?; // Watch path at first since the file may not exist yet
        let (window, preview) = self.windows.get_mut(id);

        match scroll {
            InitScroll::Fragment(hash) => {
                let scroll = ScrollRequest::Fragment(&hash);
                window.send_message(MessageToWindow::Scroll { scroll })?;
            }
            InitScroll::Heading(index) => {
                let scroll = ScrollRequest::Heading(index);
                window.send_message(MessageToWindow::Scroll { scroll })?;
            }
            InitScroll::Nop => {}
        }

        if preview.show(&path, window)? {
            self.history.push(path);
        }

        Ok(window)
    }

    fn open_window(&mut self, file: InitFile) {
        log::debug!("Open new window with file: {file:?}");
        self.init_files.push_back(file);
        self.renderer.create_window();
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

    fn pick_files(&mut self, id: R::WindowId) -> Result<()> {
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
        self.open_preview(id, last.into())?;

        Ok(())
    }

    fn pick_dirs(&mut self, id: R::WindowId) -> Result<()> {
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

    fn pick_files_in_new_window(&mut self, id: R::WindowId) {
        let (window, _) = self.windows.get(id);

        let files = self.dialog.pick_files(&window.handles());
        #[cfg(target_os = "windows")]
        let files: Vec<_> = files.into_iter().flat_map(|p| p.canonicalize().ok()).collect(); // Ensure \\? at the head of the path

        log::debug!("{} files were chosen by dialog in new windows", files.len());
        for file in files {
            self.open_window(file.into());
        }
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
        self.opener.open(&path).with_context(|| format!("Could not open config file {path:?}"))
    }

    fn is_markdown_file(&self, path: &Path) -> bool {
        self.config.watch().file_extensions.matches(path)
            && path.metadata().map(|md| !md.is_dir()).unwrap_or(false)
    }

    fn duplicate_window(&mut self, id: R::WindowId, scroll: InitScroll) {
        let (_, preview) = self.windows.get(id);
        if preview.is_empty() {
            self.renderer.create_window();
        } else {
            self.open_window(InitFile { path: preview.path().into(), scroll });
        }
    }

    fn close_window(&mut self, id: R::WindowId) -> RenderingFlow {
        log::debug!("Close window {id:?}");
        if self.windows.is_last(id) {
            self.quit(id)
        } else {
            if self.windows.remove(id).is_none() {
                log::error!("Window was closed but it was not managed by Shiba: {id:?}");
            }
            RenderingFlow::Continue
        }
    }

    fn close_other_windows(&mut self, id: R::WindowId) {
        log::debug!("Close all windows other than {id:?}");
        for (id, _, _) in self.windows.remove_others(id) {
            log::debug!("Close window: {id:?}");
        }
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

                if let Some(file) = self.init_files.pop_front() {
                    self.open_preview(id, file)?;
                } else {
                    window.send_message(MessageToWindow::Welcome)?;
                }

                #[cfg(feature = "__sanity")]
                SanityTest::new(self.renderer.clone()).run(id);
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
            FileDialog => self.pick_files(id)?,
            FileDialogNewWindow => self.pick_files_in_new_window(id),
            DirDialog => self.pick_dirs(id)?,
            OpenFile { path } => {
                self.open_preview(id, PathBuf::from(path).into())?;
            }
            ZoomIn => self.zoom(id, true)?,
            ZoomOut => self.zoom(id, false)?,
            DragWindow => self.windows.get(id).0.drag_window()?,
            ToggleMaximized => self.toggle_maximized(id),
            ToggleMinimized => self.toggle_minimized(id),
            NewWindow { path: None } => self.renderer.create_window(),
            NewWindow { path: Some(path) } => self.open_window(PathBuf::from(path).into()),
            DuplicateWindow { heading: None } => self.duplicate_window(id, InitScroll::Nop),
            DuplicateWindow { heading: Some(index) } => {
                self.duplicate_window(id, InitScroll::Heading(index));
            }
            CloseWindow => return Ok(self.close_window(id)),
            CloseAllOtherWindows => self.close_other_windows(id),
            Quit => return Ok(self.quit(id)),
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
            Quit => return Ok(self.quit(id)),
            Forward => self.navigate(id, Direction::Forward)?,
            Back => self.navigate(id, Direction::Back)?,
            Top => self.navigate(id, Direction::Top)?,
            Reload => self.reload(id)?,
            OpenFiles => self.pick_files(id)?,
            OpenFilesInNewWindow => self.pick_files_in_new_window(id),
            WatchDirs => self.pick_dirs(id)?,
            Search => self.windows.get(id).0.send_message(MessageToWindow::Search)?,
            SearchNext => self.windows.get(id).0.send_message(MessageToWindow::SearchNext)?,
            SearchPrevious => {
                self.windows.get(id).0.send_message(MessageToWindow::SearchPrevious)?
            }
            Outline => self.windows.get(id).0.send_message(MessageToWindow::Outline)?,
            Print => {
                let (window, preview) = self.windows.get(id);
                if !preview.is_empty() {
                    window.print()?;
                }
            }
            ZoomIn => self.zoom(id, true)?,
            ZoomOut => self.zoom(id, false)?,
            #[cfg(not(target_os = "macos"))]
            ToggleMenuBar => self.windows.get_mut(id).0.toggle_menu()?,
            History => self.history.send_paths(self.windows.get(id).0)?,
            ToggleAlwaysOnTop => self.toggle_always_on_top(id)?,
            ToggleMinimizeWindow => self.toggle_minimized(id),
            ToggleMaximizeWindow => self.toggle_maximized(id),
            NewWindow => self.renderer.create_window(),
            DuplicateWindow => self.duplicate_window(id, InitScroll::Nop),
            CloseWindow => return Ok(self.close_window(id)),
            CloseAllOtherWindows => self.close_other_windows(id),
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
                self.open_preview(id, path.into())?;
            }
            Event::WatchedFilesChanged(paths) => self.handle_file_changes(paths)?,
            Event::OpenLocalFile { mut file, id } => {
                if let Some(abs_path) = self.history.absolute_path(&file.path) {
                    file.path = abs_path;
                }
                if self.is_markdown_file(&file.path) {
                    log::debug!("Opening local markdown link clicked in WebView: {:?}", file);
                    self.open_preview(id, file)?;
                } else {
                    let InitFile { path, .. } = file;
                    log::debug!("Opening local link item clicked in WebView: {:?}", path);
                    self.opener
                        .open(&path)
                        .with_context(|| format!("Failed to open the local path {path:?}"))?;
                }
            }
            Event::OpenExternalLink(link) => {
                log::debug!("Opening external link item clicked in WebView: {:?}", link);
                self.opener
                    .open(&link)
                    .with_context(|| format!("Failed to open link {:?}", &link))?;
            }
            Event::Menu(item) => return self.handle_menu_item(item),
            Event::NewWindow { init_file: None } => self.renderer.create_window(),
            Event::NewWindow { init_file: Some(mut file) } => {
                if let Some(abs_path) = self.history.absolute_path(&file.path) {
                    file.path = abs_path;
                }
                if self.is_markdown_file(&file.path) {
                    self.open_window(file);
                } else {
                    let InitFile { path, .. } = file;
                    log::debug!("Opening local path link item clicked in WebView: {:?}", path);
                    self.opener
                        .open(&path)
                        .with_context(|| format!("Failed to open the local path {:?}", &path))?;
                }
            }
            Event::DuplicateWindow { scroll, id } => self.duplicate_window(id, scroll),
            Event::ProcessSingleton { paths } if paths.is_empty() => self.renderer.create_window(),
            Event::ProcessSingleton { paths } => {
                log::debug!("Watch paths via IPC: {:?}", paths.watched);
                for path in paths.watched {
                    self.watcher.watch(&path)?;
                }

                if let Some(path) = paths.init {
                    log::debug!("Open the initial file via IPC in existing window: {path:?}");
                    let id = self.windows.focused_id();
                    self.open_preview(id, path.into())?.focus();
                }

                log::debug!("Open additional windows via IPC: {:?}", paths.additional_windows);
                for path in paths.additional_windows {
                    self.open_window(path.into());
                }
            }
            Event::Error(err) => return Err(err),
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

    fn quit(&mut self, id: R::WindowId) -> RenderingFlow {
        let (window, _) = self.windows.get(id);
        window.hide();
        if let Err(error) = self.save(window) {
            self.alert("Error while the application quits", error);
        }
        log::debug!("Quit application with window {:?} and exit status {}", id, self.exit_status);
        RenderingFlow::Exit(self.exit_status)
    }

    fn handle_window_event(
        &mut self,
        id: R::WindowId,
        event: WindowEvent<R::Window>,
    ) -> Result<RenderingFlow> {
        match event {
            WindowEvent::Created(window) => {
                self.windows.add(id, window);
                // Ensure IPC messages are received after the first window is created
                if self.singleton.can_listen() {
                    self.singleton.listen(self.renderer.clone())?;
                }
            }
            WindowEvent::Minimized(is_minimized) => {
                self.windows.get_mut(id).0.save_memory(is_minimized)?
            }
            WindowEvent::Focused => self.windows.set_focus(id),
            WindowEvent::Closed => return Ok(self.close_window(id)),
        }
        Ok(RenderingFlow::Continue)
    }

    fn alert(&mut self, title: &'static str, error: Error) {
        log::error!("Error while handling window event: {title}: {error}");
        let handles = if self.windows.is_empty() {
            WindowHandles::unavailable() // Error may happen after all windows are closed
        } else {
            self.windows.focused().0.handles()
        };
        self.dialog.alert(&error.context(title), &handles);
        self.exit_status = 1;
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
        if self.windows.is_empty() {
            log::error!("Ignore the event because no window exists: {event:?}");
            return RenderingFlow::Continue;
        }
        self.handle_event(event).unwrap_or_else(|err| {
            self.alert("Could not handle application event", err);
            RenderingFlow::Continue
        })
    }

    fn on_window(&mut self, id: Self::WindowId, event: WindowEvent<Self::Window>) -> RenderingFlow {
        if self.windows.is_empty() && !matches!(event, WindowEvent::Created(_)) {
            log::error!(
                "Ignore the window event for window {id:?} because no window exists: {event:?}",
            );
            return RenderingFlow::Continue;
        }
        self.handle_window_event(id, event).unwrap_or_else(|err| {
            self.alert("Could not handle window event", err);
            RenderingFlow::Continue
        })
    }
}
