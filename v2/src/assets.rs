use std::mem;

#[cfg(debug_assertions)]
pub const BUNDLE_JS: &[u8] = include_bytes!("assets/bundle.js");
#[cfg(not(debug_assertions))]
pub const BUNDLE_JS: &[u8] = include_bytes!("assets/bundle.min.js");
pub const INDEX_HTML: &[u8] = include_bytes!("assets/index.html");
pub const GITHUB_MARKDOWN_CSS: &[u8] = include_bytes!("assets/github-markdown.css");
pub const STYLE_CSS: &[u8] = include_bytes!("assets/style.css");
pub const HLJS_GITHUB_CSS: &[u8] = include_bytes!("assets/github.css");

// TODO: hljs-theme.css will be customizable with user configuration file
// TODO: user css can be applied by user configuration file

pub fn assets(path: &str) -> (Vec<u8>, &'static str) {
    let mime = if path.ends_with('/') || path.ends_with(".html") {
        "text/html;charset=UTF-8"
    } else if path.ends_with(".js") {
        "text/javascript;charset=UTF-8"
    } else if path.ends_with(".css") {
        "text/css;charset=UTF-8"
    } else {
        "text/plain;charset=UTF-8"
    };

    #[rustfmt::skip]
    let body = match path {
        "/"                    => INDEX_HTML,
        "/bundle.js"           => BUNDLE_JS,
        "/style.css"           => STYLE_CSS,
        "/github-markdown.css" => GITHUB_MARKDOWN_CSS,
        "/hljs-theme.css"      => HLJS_GITHUB_CSS,
        _                      => &[],
    };

    // Response body of custom protocol handler requires `Vec<u8>`
    (body.to_vec(), mime)
}

const ASSET_PATHS: &[&str] =
    &["/", "/bundle.js", "/style.css", "/github-markdown.css", "/hljs-theme.css"];

#[derive(Default)]
pub struct AssetsLoaded([bool; ASSET_PATHS.len()]);

impl AssetsLoaded {
    pub fn is_loaded(&mut self, path: &str) -> bool {
        let Some(idx) = ASSET_PATHS.iter().position(|&p| p == path) else { return false; };
        mem::replace(&mut self.0[idx], true)
    }
}

pub fn is_asset_path(path: &str) -> bool {
    ASSET_PATHS.contains(&path)
}
