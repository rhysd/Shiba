use crate::renderer::Theme;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(debug_assertions)]
pub const BUNDLE_JS: &[u8] = include_bytes!("assets/bundle.js");
#[cfg(not(debug_assertions))]
pub const BUNDLE_JS: &[u8] = include_bytes!("assets/bundle.min.js");
pub const INDEX_HTML: &[u8] = include_bytes!("assets/index.html");
pub const GITHUB_MARKDOWN_CSS: &[u8] =
    include_bytes!("assets/node_modules/github-markdown-css/github-markdown.css");
pub const STYLE_CSS: &[u8] = include_bytes!("assets/web/style.css");
pub const HLJS_GITHUB_CSS: &[u8] =
    include_bytes!("assets/node_modules/highlight.js/styles/github.css");
pub const HLJS_GITHUB_DARK_CSS: &[u8] =
    include_bytes!("assets/node_modules/highlight.js/styles/github-dark.css");
pub const TIPPY_CSS: &[u8] = include_bytes!("assets/node_modules/tippy.js/dist/tippy.css");
pub const TIPPY_LIGHT_CSS: &[u8] = include_bytes!("assets/node_modules/tippy.js/themes/light.css");
pub const LOGO_PNG: &[u8] = include_bytes!("assets/logo.png");

// TODO: hljs-theme.css will be customizable with user configuration file
// TODO: user css can be applied by user configuration file

const ONETIME_ASSET_PATHS: &[&str] = &[
    "/",
    "/bundle.js",
    "/style.css",
    "/github-markdown.css",
    "/hljs-theme.css",
    "/tippy.css",
    "/tippy-light.css",
];

const OTHER_ASSET_PATHS: &[&str] = &["/logo.png"];

#[derive(Default)]
pub struct Assets {
    loaded: [AtomicBool; ONETIME_ASSET_PATHS.len()],
}

impl Assets {
    pub fn load(path: &str, theme: Theme) -> (&'static [u8], &'static str) {
        let mime = if path.ends_with('/') || path.ends_with(".html") {
            "text/html;charset=UTF-8"
        } else if path.ends_with(".js") {
            "text/javascript;charset=UTF-8"
        } else if path.ends_with(".css") {
            "text/css;charset=UTF-8"
        } else if path.ends_with(".png") {
            "image/png"
        } else {
            "text/plain;charset=UTF-8"
        };

        #[rustfmt::skip]
        let body = match path {
            "/"                                       => INDEX_HTML,
            "/bundle.js"                              => BUNDLE_JS,
            "/style.css"                              => STYLE_CSS,
            "/github-markdown.css"                    => GITHUB_MARKDOWN_CSS,
            "/hljs-theme.css" if theme == Theme::Dark => HLJS_GITHUB_DARK_CSS,
            "/hljs-theme.css"                         => HLJS_GITHUB_CSS,
            "/tippy.css"                              => TIPPY_CSS,
            "/tippy-light.css"                        => TIPPY_LIGHT_CSS,
            "/logo.png"                               => LOGO_PNG,
            _                                         => unreachable!(),
        };

        (body, mime)
    }

    // `&self` must not be `&mut self` since the handler callbacks of `WebViewBuilder` are defined as `Fn`.
    // The `Fn` boundary is derived from `webkit2gtk::WebView::connect_decide_policy` so it is difficult to change.
    // https://github.com/tauri-apps/webkit2gtk-rs/blob/cce947f86f2c0d50710c1ea9ea9f160c8b6cbf4a/src/auto/web_view.rs#L1249
    pub fn is_asset(&self, path: &str) -> bool {
        if OTHER_ASSET_PATHS.contains(&path) {
            return true;
        }
        let Some(idx) = ONETIME_ASSET_PATHS.iter().position(|&p| p == path) else { return false; };
        let loaded = self.loaded[idx].swap(true, Ordering::Relaxed);
        !loaded
    }
}
