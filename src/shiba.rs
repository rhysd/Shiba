use crate::cli::Options;
use crate::config::Config;
use crate::dialog::Dialog;
use crate::history::{Direction, History};
use crate::opener::Opener;
use crate::preview::Preview;
use crate::renderer::{
    Event, EventHandler, MenuItem, MessageFromWindow, MessageToWindow, Renderer, RenderingFlow,
    Window, WindowHandles,
};
#[cfg(feature = "__sanity")]
use crate::sanity::SanityTest;
use crate::watcher::{PathFilter, Watcher};
use anyhow::{Context as _, Error, Result};
use std::mem;
use std::path::PathBuf;

pub struct Shiba<R: Renderer, O, W, D> {
    window: R::Window, // Only a single window is currently supported
    opener: O,
    history: History,
    watcher: W,
    dialog: D,
    config: Config,
    preview: Preview,
    init_file: Option<PathBuf>,
    #[cfg(feature = "__sanity")]
    sanity: SanityTest<R::EventSender>,
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
        fn on_err<D: Dialog>(err: Error) -> Error {
            let err = err.context("Could not launch application");
            if let Ok(dialog) = D::new(&Config::default()) {
                dialog.alert(&err, &WindowHandles::unsupported());
            }
            err
        }
        let mut renderer = R::new().map_err(on_err::<D>)?;
        let dog = Self::new(options, &mut renderer).map_err(on_err::<D>)?;
        renderer.start(dog)
    }

    pub fn new(mut options: Options, renderer: &mut R) -> Result<Self> {
        log::debug!("Application options: {:?}", options);
        let watch_paths = mem::take(&mut options.watch_paths);
        let init_file = mem::take(&mut options.init_file);

        let config = Config::load(options)?;
        log::debug!("Application config: {:?}", config);

        let window = renderer.create_window(&config)?;
        log::debug!("Created window with ID: {:?}", window.id());

        let filter = PathFilter::new(config.watch());
        let mut watcher = W::new(renderer.create_sender(), filter)?;
        let mut history = History::load(&config);
        for path in watch_paths {
            log::debug!("Watching initial path: {:?}", path);
            watcher.watch(&path)?;
            history.push(path);
        }

        Ok(Self {
            window,
            opener: O::default(),
            history,
            watcher,
            dialog: D::new(&config)?,
            config,
            preview: Preview::default(),
            init_file,
            #[cfg(feature = "__sanity")]
            sanity: SanityTest::new(renderer.create_sender()),
        })
    }

    fn open_preview(&mut self, path: PathBuf) -> Result<()> {
        self.watcher.watch(&path)?; // Watch path at first since the file may not exist yet
        if self.preview.show(&path, &self.window)? {
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
            if self.preview.show(path, &self.window)? {
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
            self.preview.show(path, &self.window)?;
            self.window.send_message(MessageToWindow::Reload)?;
        }
        Ok(())
    }

    fn open_files(&mut self) -> Result<()> {
        #[cfg_attr(target_os = "windows", allow(unused_mut))]
        let mut files = self.dialog.pick_files(&self.window.handles());
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
        let dirs = self.dialog.pick_dirs(&self.window.handles());
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
            self.window.zoom_level().zoom_in()
        } else {
            self.window.zoom_level().zoom_out()
        };

        let Some(level) = level else {
            return Ok(());
        };

        self.window.zoom(level)?;
        let percent = level.percent();
        log::debug!("Changed zoom factor: {}%", percent);
        self.window.send_message(MessageToWindow::Zoomed { percent })?;

        Ok(())
    }

    fn toggle_always_on_top(&mut self) -> Result<()> {
        let pinned = !self.window.always_on_top();
        log::debug!("Toggle always-on-top (pinned={})", pinned);
        self.window.set_always_on_top(pinned);
        self.window.send_message(MessageToWindow::AlwaysOnTop { pinned })
    }

    fn toggle_maximized(&mut self) {
        let maximized = !self.window.is_maximized();
        log::debug!("Toggle maximized window (maximized={})", maximized);
        self.window.maximize(maximized);
    }

    fn toggle_minimized(&mut self) {
        let minimized = !self.window.is_minimized();
        log::debug!("Toggle minimized window (minimized={})", minimized);
        self.window.minimize(minimized);
    }

    fn open_config(&mut self) -> Result<()> {
        let path = self.config.config_file()?;
        log::debug!("Opening config file via menu item: {:?}", path);
        self.opener.open(&path)
    }

    fn handle_window_message(&mut self, message: MessageFromWindow) -> Result<RenderingFlow> {
        use MessageFromWindow::*;
        match message {
            Init => {
                if self.config.debug() {
                    self.window.send_message(MessageToWindow::Debug)?;
                }

                self.window.send_message(MessageToWindow::Config {
                    keymaps: self.config.keymaps(),
                    search: self.config.search(),
                    home: self.preview.home_dir(),
                    window: self.window.appearance(),
                })?;

                // Open window when the content is ready. Otherwise a white window flashes when dark theme.
                self.window.show();

                if let Some(path) = mem::take(&mut self.init_file) {
                    self.open_preview(path)?;
                } else {
                    self.window.send_message(MessageToWindow::Welcome)?;
                }

                #[cfg(feature = "__sanity")]
                self.sanity.run_test();
            }
            Search { query, index, matcher } => {
                self.preview.search(&self.window, &query, index, matcher)?;
            }
            GoForward => self.navigate(Direction::Forward)?,
            GoBack => self.navigate(Direction::Back)?,
            GoTop if self.preview.is_empty() => self.navigate(Direction::Back)?,
            GoTop => self.navigate(Direction::Top)?,
            History => self.history.send_paths(&self.window)?,
            Reload => self.reload()?,
            FileDialog => self.open_files()?,
            DirDialog => self.open_dirs()?,
            OpenFile { path } => self.open_preview(PathBuf::from(path))?,
            ZoomIn => self.zoom(true)?,
            ZoomOut => self.zoom(false)?,
            DragWindow => self.window.drag_window()?,
            ToggleMaximized => self.toggle_maximized(),
            ToggleMinimized => self.toggle_minimized(),
            Quit => return Ok(RenderingFlow::Exit),
            OpenMenu { position } => self.window.show_menu_at(position),
            ToggleMenuBar => self.window.toggle_menu()?,
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
            Quit => return Ok(RenderingFlow::Exit),
            Forward => self.navigate(Direction::Forward)?,
            Back => self.navigate(Direction::Back)?,
            Top if self.preview.is_empty() => self.navigate(Direction::Back)?,
            Top => self.navigate(Direction::Top)?,
            Reload => self.reload()?,
            OpenFiles => self.open_files()?,
            WatchDirs => self.open_dirs()?,
            Search => self.window.send_message(MessageToWindow::Search)?,
            SearchNext => self.window.send_message(MessageToWindow::SearchNext)?,
            SearchPrevious => self.window.send_message(MessageToWindow::SearchPrevious)?,
            Outline => self.window.send_message(MessageToWindow::Outline)?,
            Print if self.preview.is_empty() => {}
            Print => self.window.print()?,
            ZoomIn => self.zoom(true)?,
            ZoomOut => self.zoom(false)?,
            #[cfg(not(target_os = "macos"))]
            ToggleMenuBar => self.window.toggle_menu()?,
            History => self.history.send_paths(&self.window)?,
            ToggleAlwaysOnTop => self.toggle_always_on_top()?,
            ToggleMinimizeWindow => self.toggle_minimized(),
            ToggleMaximizeWindow => self.toggle_maximized(),
            Help => self.window.send_message(MessageToWindow::Help)?,
            OpenRepo => self.opener.open("https://github.com/rhysd/Shiba")?,
            EditConfig => self.open_config()?,
            DeleteHistory => {
                if self.dialog.yes_no(
                    "Deleting history...",
                    "Are you sure you want to delete all history?",
                    &self.window.handles(),
                ) {
                    self.history.clear(&self.config)?;
                    self.window.delete_cache()?;
                }
            }
        }
        Ok(RenderingFlow::Continue)
    }

    fn handle_event(&mut self, event: Event) -> Result<RenderingFlow> {
        log::debug!("Handling event {:?}", event);
        match event {
            Event::WindowMessage(msg) => return self.handle_window_message(msg),
            Event::FileDrop(mut path) => {
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
                    self.preview.show(current, &self.window)?;
                    return Ok(RenderingFlow::Continue);
                }
                // Choose the last one to preview if the current file is not included in `paths`
                if let Some(mut path) = paths.pop() {
                    if !path.is_absolute() {
                        path = path.canonicalize()?;
                    }
                    if self.preview.show(&path, &self.window)? {
                        self.history.push(path);
                    }
                }
            }
            Event::OpenLocalPath(mut path) => {
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
            Event::Minimized(is_minimized) => self.window.save_memory(is_minimized)?,
            Event::Error(err) => self.dialog.alert(&err, &self.window.handles()),
        }
        Ok(RenderingFlow::Continue)
    }

    fn shutdown(&mut self) -> Result<()> {
        log::debug!("Handling application exit");

        // Hide the window before destroying it to avoid flickering.
        self.window.hide();

        if self.config.window().restore
            && let Some(state) = self.window.state()
            && state.height > 0.0
            && state.width > 0.0
        {
            log::debug!("Saving window state as persistent data: {:?}", state);
            self.config.data_dir().save(&state)?;
        }
        self.history.save(&self.config)?;
        Ok(())
    }
}

impl<R, O, W, D> EventHandler for Shiba<R, O, W, D>
where
    R: Renderer,
    O: Opener,
    W: Watcher,
    D: Dialog,
{
    fn on_event(&mut self, event: Event) -> RenderingFlow {
        self.handle_event(event).unwrap_or_else(|err| {
            let err = err.context("Could not handle event");
            self.dialog.alert(&err, &self.window.handles());
            RenderingFlow::Continue
        })
    }

    fn on_exit(&mut self) -> i32 {
        if let Err(err) = self.shutdown() {
            let err = err.context("Could not shutdown application");
            // Don't pass window handles because the window is already hidden in `self.shutdown` call.
            self.dialog.alert(&err, &WindowHandles::unsupported());
            1
        } else {
            0
        }
    }
}
