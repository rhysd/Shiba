use crate::cli::Options;
use crate::config::Config;
use crate::dialog::Dialog;
use crate::history::History;
use crate::opener::Opener;
use crate::preview::Preview;
use crate::renderer::{
    Event, EventHandler, MenuItem, MessageFromRenderer, MessageToRenderer, Renderer, Rendering,
    RenderingFlow,
};
#[cfg(feature = "__sanity")]
use crate::sanity::SanityTest;
use crate::watcher::{PathFilter, Watcher};
use anyhow::{Context as _, Error, Result};
use std::marker::PhantomData;
use std::mem;
use std::path::PathBuf;

enum Zoom {
    In,
    Out,
}

pub struct Shiba<R: Rendering, O, W, D> {
    renderer: R::Renderer,
    opener: O,
    history: History,
    watcher: W,
    config: Config,
    preview: Preview,
    init_file: Option<PathBuf>,
    _dialog: PhantomData<D>,
    #[cfg(feature = "__sanity")]
    sanity: SanityTest<R::EventSender>,
}

impl<R, O, W, D> Shiba<R, O, W, D>
where
    R: Rendering,
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
            D::alert(&err);
            err
        }
        let mut rendering = R::new().map_err(on_err::<D>)?;
        let dog = Self::new(options, &mut rendering).map_err(on_err::<D>)?;
        rendering.start(dog)
    }

    pub fn new(mut options: Options, rendering: &mut R) -> Result<Self> {
        log::debug!("Application options: {:?}", options);
        let watch_paths = mem::take(&mut options.watch_paths);
        let init_file = mem::take(&mut options.init_file);

        let config = Config::load(options)?;
        log::debug!("Application config: {:?}", config);

        let renderer = rendering.create_renderer(&config)?;

        let filter = PathFilter::new(config.watch());
        let mut watcher = W::new(rendering.create_sender(), filter)?;
        let mut history = History::load(&config);
        for path in watch_paths {
            log::debug!("Watching initial path: {:?}", path);
            watcher.watch(&path)?;
            history.push(path);
        }

        Ok(Self {
            renderer,
            opener: O::default(),
            history,
            watcher,
            config,
            preview: Preview::default(),
            init_file,
            _dialog: PhantomData,
            #[cfg(feature = "__sanity")]
            sanity: SanityTest::new(rendering.create_sender()),
        })
    }

    fn preview(&mut self, path: PathBuf) -> Result<()> {
        self.watcher.watch(&path)?; // Watch path at first since the file may not exist yet
        if self.preview.show(&path, &self.renderer)? {
            self.history.push(path);
        }
        Ok(())
    }

    fn forward(&mut self) -> Result<()> {
        if let Some(path) = self.history.forward() {
            log::debug!("Forward to next preview page: {:?}", path);
            self.preview.show(path, &self.renderer)?;
        }
        Ok(())
    }

    fn back(&mut self) -> Result<()> {
        if let Some(path) = self.history.back() {
            log::debug!("Back to previous preview page: {:?}", path);
            self.preview.show(path, &self.renderer)?;
        }
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
            self.preview.show(path, &self.renderer)?;
            self.renderer.send_message(MessageToRenderer::Reload)?;
        }
        Ok(())
    }

    fn open_files(&mut self) -> Result<()> {
        let extensions = self.config.watch().file_extensions();
        let dir = self.config.dialog().default_dir()?;
        #[cfg_attr(target_os = "windows", allow(unused_mut))]
        let mut files = D::pick_files(&dir, extensions);
        #[cfg(target_os = "windows")]
        let mut files: Vec<_> = files.into_iter().flat_map(|p| p.canonicalize().ok()).collect(); // Ensure \\? at the head of the path

        let Some(last) = files.pop() else {
            log::debug!("No file was chosen by dialog");
            return Ok(());
        };

        log::debug!("{} files were chosen by dialog", files.len());
        for file in files {
            self.watcher.watch(&file)?;
            self.history.push(file);
        }
        log::debug!("Previewing the last file chosen by dialog: {last:?}");
        self.preview(last)?;

        Ok(())
    }

    fn open_dirs(&mut self) -> Result<()> {
        let dir = self.config.dialog().default_dir()?;
        let dirs = D::pick_dirs(&dir);
        #[cfg(target_os = "windows")]
        let dirs: Vec<_> = dirs.into_iter().flat_map(|p| p.canonicalize().ok()).collect(); // Ensure \\? at the head of the path

        log::debug!("{} directories were chosen by dialog", dirs.len());
        for dir in dirs {
            log::debug!("Watching a directory chosen by dialog: {:?}", dir);
            self.watcher.watch(&dir)?;
        }

        Ok(())
    }

    fn zoom(&mut self, zoom: Zoom) -> Result<()> {
        let level = match zoom {
            Zoom::In => self.renderer.zoom_level().zoom_in(),
            Zoom::Out => self.renderer.zoom_level().zoom_out(),
        };

        let Some(level) = level else {
            return Ok(());
        };

        self.renderer.zoom(level)?;
        let percent = level.percent();
        log::debug!("Changed zoom factor: {}%", percent);
        self.renderer.send_message(MessageToRenderer::Zoomed { percent })?;

        Ok(())
    }

    fn toggle_always_on_top(&mut self) -> Result<()> {
        let pinned = !self.renderer.always_on_top();
        log::debug!("Toggle always-on-top (pinned={})", pinned);
        self.renderer.set_always_on_top(pinned);
        self.renderer.send_message(MessageToRenderer::AlwaysOnTop { pinned })
    }

    fn toggle_maximized(&mut self) {
        let maximized = !self.renderer.is_maximized();
        log::debug!("Toggle maximized window (maximized={})", maximized);
        self.renderer.set_maximized(maximized);
    }

    fn open_config(&mut self) -> Result<()> {
        let path = self.config.config_file()?;
        log::debug!("Opening config file via menu item: {:?}", path);
        self.opener.open(&path)
    }

    fn handle_renderer_message(&mut self, message: MessageFromRenderer) -> Result<RenderingFlow> {
        use MessageFromRenderer::*;
        match message {
            Init => {
                if self.config.debug() {
                    self.renderer.send_message(MessageToRenderer::Debug)?;
                }

                self.renderer.send_message(MessageToRenderer::Config {
                    keymaps: self.config.keymaps(),
                    search: self.config.search(),
                    home: self.preview.home_dir(),
                    window: self.renderer.window_appearance(),
                })?;

                // Open window when the content is ready. Otherwise a white window flashes when dark theme.
                self.renderer.show();

                if let Some(path) = mem::take(&mut self.init_file) {
                    self.preview(path)?;
                } else {
                    self.renderer.send_message(MessageToRenderer::Welcome)?;
                }

                #[cfg(feature = "__sanity")]
                self.sanity.run_test();
            }
            Search { query, index, matcher } => {
                self.preview.search(&self.renderer, &query, index, matcher)?;
            }
            Forward => self.forward()?,
            Back => self.back()?,
            History => self.history.send_paths(&self.renderer)?,
            Reload => self.reload()?,
            FileDialog => self.open_files()?,
            DirDialog => self.open_dirs()?,
            OpenFile { path } => self.preview(PathBuf::from(path))?,
            ZoomIn => self.zoom(Zoom::In)?,
            ZoomOut => self.zoom(Zoom::Out)?,
            DragWindow => self.renderer.drag_window()?,
            ToggleMaximized => self.toggle_maximized(),
            Quit => return Ok(RenderingFlow::Exit),
            OpenMenu { position } => self.renderer.show_menu_at(position),
            ToggleMenuBar => self.renderer.toggle_menu()?,
            Error { message } => anyhow::bail!("Error reported from renderer: {}", message),
        }
        Ok(RenderingFlow::Continue)
    }

    fn handle_menu_item(&mut self, item: MenuItem) -> Result<RenderingFlow> {
        use MenuItem::*;

        log::debug!("Menu item was clicked: {:?}", item);
        match item {
            Quit => return Ok(RenderingFlow::Exit),
            Forward => self.forward()?,
            Back => self.back()?,
            Reload => self.reload()?,
            OpenFiles => self.open_files()?,
            WatchDirs => self.open_dirs()?,
            Search => self.renderer.send_message(MessageToRenderer::Search)?,
            SearchNext => self.renderer.send_message(MessageToRenderer::SearchNext)?,
            SearchPrevious => self.renderer.send_message(MessageToRenderer::SearchPrevious)?,
            Outline => self.renderer.send_message(MessageToRenderer::Outline)?,
            Print => self.renderer.print()?,
            ZoomIn => self.zoom(Zoom::In)?,
            ZoomOut => self.zoom(Zoom::Out)?,
            #[cfg(not(target_os = "macos"))]
            ToggleMenuBar => self.renderer.toggle_menu()?,
            History => self.history.send_paths(&self.renderer)?,
            ToggleAlwaysOnTop => self.toggle_always_on_top()?,
            Help => self.renderer.send_message(MessageToRenderer::Help)?,
            OpenRepo => self.opener.open("https://github.com/rhysd/Shiba")?,
            EditConfig => self.open_config()?,
            DeleteCookies => self.renderer.delete_cookies()?,
        }
        Ok(RenderingFlow::Continue)
    }

    fn handle_event(&mut self, event: Event) -> Result<RenderingFlow> {
        log::debug!("Handling event {:?}", event);
        match event {
            Event::RendererMessage(msg) => return self.handle_renderer_message(msg),
            Event::FileDrop(mut path) => {
                log::debug!("Previewing file dropped into window: {:?}", path);
                if !path.is_absolute() {
                    path = path.canonicalize()?;
                }
                self.preview(path)?;
            }
            Event::WatchedFilesChanged(mut paths) => {
                log::debug!("Files changed: {:?}", paths);
                if let Some(current) = self.history.current()
                    && paths.contains(current)
                {
                    self.preview.show(current, &self.renderer)?;
                    return Ok(RenderingFlow::Continue);
                }
                // Choose the last one to preview if the current file is not included in `paths`
                if let Some(mut path) = paths.pop() {
                    if !path.is_absolute() {
                        path = path.canonicalize()?;
                    }
                    if self.preview.show(&path, &self.renderer)? {
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
                let is_markdown = self.config.watch().file_extensions().matches(&path);
                if is_markdown {
                    log::debug!("Opening local markdown link clicked in WebView: {:?}", path);
                    self.preview(path)?;
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
            Event::Minimized(is_minimized) => self.renderer.save_memory(is_minimized)?,
            Event::Error(err) => D::alert(&err),
        }
        Ok(RenderingFlow::Continue)
    }

    fn shutdown(&mut self) -> Result<()> {
        log::debug!("Handling application exit");

        // Hide the window before destroying it to avoid flickering.
        self.renderer.hide();

        if self.config.window().restore
            && let Some(state) = self.renderer.window_state()
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
    R: Rendering,
    O: Opener,
    W: Watcher,
    D: Dialog,
{
    fn on_event(&mut self, event: Event) -> RenderingFlow {
        self.handle_event(event).unwrap_or_else(|err| {
            D::alert(&err.context("Could not handle event"));
            RenderingFlow::Continue
        })
    }

    fn on_exit(&mut self) -> i32 {
        if let Err(err) = self.shutdown() {
            D::alert(&err.context("Could not shutdown application"));
            1
        } else {
            0
        }
    }
}
