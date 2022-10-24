use anyhow::Result;
use getopts::Options;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use wry::application::accelerator::Accelerator;
use wry::application::event::{Event, StartCause, WindowEvent};
use wry::application::event_loop::{ControlFlow, EventLoop};
use wry::application::keyboard::{KeyCode, ModifiersState};
use wry::application::menu::{AboutMetadata, MenuBar, MenuItem, MenuItemAttributes};
use wry::application::window::WindowBuilder;
use wry::webview::{FileDropEvent, WebView, WebViewBuilder};

const HTML: &str = include_str!("bundle.html");

#[derive(Serialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
enum MessageToWebView<'a> {
    Content { content: &'a str },
}

impl<'a> MessageToWebView<'a> {
    fn send_to(&self, webview: &WebView) -> Result<()> {
        let mut buf = b"window.myMarkdownPreview.receive(".to_vec();
        serde_json::to_writer(&mut buf, self)?;
        buf.push(b')');
        webview.evaluate_script(&String::from_utf8(buf).unwrap())?; // XXX: This UTF-8 validation is redundant
        Ok(())
    }

    fn preview(path: impl AsRef<Path>, webview: &WebView) -> Result<()> {
        let path = path.as_ref();
        log::debug!("Opening markdown preview for {:?}", path);
        let content = fs::read_to_string(path)?;
        let msg = MessageToWebView::Content { content: &content };
        msg.send_to(webview)
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
enum MessageFromWebView {
    Init,
    Open { link: String },
}

#[derive(Debug)]
enum UserEvent {
    FromWebView(MessageFromWebView),
    FileDrop(PathBuf),
}

fn usage(options: Options) {
    let program = env::args().next().unwrap();
    let header = format!("Usage: {} [option] FILE", program);
    println!("{}", options.usage(&header));
}

fn main() -> Result<()> {
    let debug = env::var("DEBUG").is_ok();
    let level = if debug { log::LevelFilter::Debug } else { log::LevelFilter::Info };

    env_logger::builder().filter_level(level).init();

    let mut options = Options::new();
    options.optflag("h", "help", "print this help");
    let matches = options.parse(env::args().skip(1))?;
    if matches.opt_present("h") {
        usage(options);
        return Ok(());
    }

    let event_loop = EventLoop::with_user_event();
    let ipc_proxy = event_loop.create_proxy();
    let file_drop_proxy = event_loop.create_proxy();

    let window = WindowBuilder::new().with_title("Markdown Preview").build(&event_loop)?;
    log::debug!("Event loop and window were created successfully");

    let mut menu = MenuBar::new();
    let mut sub_menu = MenuBar::new();
    sub_menu
        .add_native_item(MenuItem::About("Markdown Preview".to_string(), AboutMetadata::default()));
    let quit_item = sub_menu.add_item(
        MenuItemAttributes::new("Quit")
            .with_accelerators(&Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyQ)),
    );
    let quit_item = quit_item.id();
    menu.add_submenu("File", true, sub_menu);
    window.set_menu(Some(menu));
    log::debug!("Added menubar to window (quit={:?})", quit_item);

    let webview = WebViewBuilder::new(window)?
        .with_html(HTML)?
        .with_devtools(debug)
        .with_ipc_handler(move |_w, s| {
            let m: MessageFromWebView = serde_json::from_str(&s).unwrap();
            log::debug!("Message from WebView: {:?}", m);
            if let Err(e) = ipc_proxy.send_event(UserEvent::FromWebView(m)) {
                log::error!("Could not send user event for message from WebView: {}", e);
            }
        })
        .with_file_drop_handler(move |_w, e| {
            if let FileDropEvent::Dropped(paths) = e {
                log::debug!("Files were dropped (the first one will be opened): {:?}", paths);
                if let Some(path) = paths.into_iter().next() {
                    if let Err(e) = file_drop_proxy.send_event(UserEvent::FileDrop(path)) {
                        log::error!("Could not send user event for file drop: {}", e);
                    }
                }
            }
            true
        })
        .build()?;
    log::debug!("Created WebView successfully");

    #[cfg(debug_assertions)]
    if debug {
        webview.open_devtools(); // This method is defined in debug build only
        log::debug!("Opened DevTools for debugging");
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                log::debug!("Application has started");
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                log::debug!("Closing window was requested");
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(event) => match event {
                UserEvent::FromWebView(msg) => match msg {
                    MessageFromWebView::Init => {
                        if let Some(path) = matches.free.first() {
                            log::debug!(
                                "Opening file in preview specified via argument: {:?}",
                                path
                            );
                            if let Err(e) = MessageToWebView::preview(path, &webview) {
                                log::error!("Could not preview {:?}: {}", path, e);
                            }
                        }
                    }
                    MessageFromWebView::Open { link }
                        if link.starts_with("https://") || link.starts_with("http://") =>
                    {
                        log::debug!("Opening web link clicked in WebView: {:?}", link);
                        if let Err(e) = open::that(&link) {
                            log::error!("Could not open web link {:?}: {}", link, e);
                        }
                    }
                    MessageFromWebView::Open { mut link } => {
                        if link.starts_with("file://") {
                            link.drain(.."file://".len());
                            #[cfg(target_os = "windows")]
                            {
                                link = link.replace('/', "\\");
                            }
                        }
                        log::debug!("Opening link item clicked in WebView: {:?}", link);
                        // TODO: Open markdown document in this app
                        if let Err(e) = open::that(&link) {
                            log::error!("Could not open {:?}: {}", link, e);
                        }
                    }
                },
                UserEvent::FileDrop(path) => {
                    log::debug!("Previewing file dropped into window: {:?}", path);
                    if let Err(e) = MessageToWebView::preview(&path, &webview) {
                        log::error!("Could not preview {:?}: {}", path, e);
                    }
                }
            },
            Event::MenuEvent { menu_id, .. } if menu_id == quit_item => {
                log::debug!("'Quit' menu item was clicked");
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}
