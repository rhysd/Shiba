use crate::config::Preview as Config;
use crate::renderer::Theme as WindowTheme;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(debug_assertions)]
pub const BUNDLE_JS: &[u8] = include_bytes!("assets/bundle.js");
#[cfg(not(debug_assertions))]
pub const BUNDLE_JS: &[u8] = include_bytes!("assets/bundle.min.js");
pub const INDEX_HTML: &[u8] = include_bytes!("assets/index.html");
pub const GITHUB_MARKDOWN_CSS: &[u8] =
    include_bytes!("assets/node_modules/github-markdown-css/github-markdown.css");
pub const STYLE_CSS: &[u8] = include_bytes!("assets/web/style.css");
pub const HLJS_DEFAULT_LIGHT_CSS: &[u8] =
    include_bytes!("assets/node_modules/highlight.js/styles/github.css");
pub const HLJS_DEFAULT_DARK_CSS: &[u8] =
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

#[rustfmt::skip]
fn load_hljs_css(theme_name: &str, default: &'static [u8]) -> &'static [u8] {
    log::debug!("Loading highlight.js theme {:?}", theme_name);
    match theme_name {
        "Github" => HLJS_DEFAULT_LIGHT_CSS,
        "Github Dark" => HLJS_DEFAULT_DARK_CSS,
        "A11Y Dark"                => include_bytes!("assets/node_modules/highlight.js/styles/a11y-dark.css"),
        "A11Y Light"               => include_bytes!("assets/node_modules/highlight.js/styles/a11y-light.css"),
        "Agate"                    => include_bytes!("assets/node_modules/highlight.js/styles/agate.css"),
        "An Old Hope"              => include_bytes!("assets/node_modules/highlight.js/styles/an-old-hope.css"),
        "Androidstudio"            => include_bytes!("assets/node_modules/highlight.js/styles/androidstudio.css"),
        "Arduino Light"            => include_bytes!("assets/node_modules/highlight.js/styles/arduino-light.css"),
        "Arta"                     => include_bytes!("assets/node_modules/highlight.js/styles/arta.css"),
        "Ascetic"                  => include_bytes!("assets/node_modules/highlight.js/styles/ascetic.css"),
        "Atom One Dark Reasonable" => include_bytes!("assets/node_modules/highlight.js/styles/atom-one-dark-reasonable.css"),
        "Atom One Dark"            => include_bytes!("assets/node_modules/highlight.js/styles/atom-one-dark.css"),
        "Atom One Light"           => include_bytes!("assets/node_modules/highlight.js/styles/atom-one-light.css"),
        "Brown Paper"              => include_bytes!("assets/node_modules/highlight.js/styles/brown-paper.css"),
        "Codepen Embed"            => include_bytes!("assets/node_modules/highlight.js/styles/codepen-embed.css"),
        "Color Brewer"             => include_bytes!("assets/node_modules/highlight.js/styles/color-brewer.css"),
        "Dark"                     => include_bytes!("assets/node_modules/highlight.js/styles/dark.css"),
        "Default"                  => include_bytes!("assets/node_modules/highlight.js/styles/default.css"),
        "Devibeans"                => include_bytes!("assets/node_modules/highlight.js/styles/devibeans.css"),
        "Docco"                    => include_bytes!("assets/node_modules/highlight.js/styles/docco.css"),
        "Far"                      => include_bytes!("assets/node_modules/highlight.js/styles/far.css"),
        "Felipec"                  => include_bytes!("assets/node_modules/highlight.js/styles/felipec.css"),
        "Foundation"               => include_bytes!("assets/node_modules/highlight.js/styles/foundation.css"),
        "Github Dark Dimmed"       => include_bytes!("assets/node_modules/highlight.js/styles/github-dark-dimmed.css"),
        "Gml"                      => include_bytes!("assets/node_modules/highlight.js/styles/gml.css"),
        "Googlecode"               => include_bytes!("assets/node_modules/highlight.js/styles/googlecode.css"),
        "Gradient Dark"            => include_bytes!("assets/node_modules/highlight.js/styles/gradient-dark.css"),
        "Gradient Light"           => include_bytes!("assets/node_modules/highlight.js/styles/gradient-light.css"),
        "Grayscale"                => include_bytes!("assets/node_modules/highlight.js/styles/grayscale.css"),
        "Hybrid"                   => include_bytes!("assets/node_modules/highlight.js/styles/hybrid.css"),
        "Idea"                     => include_bytes!("assets/node_modules/highlight.js/styles/idea.css"),
        "Intellij Light"           => include_bytes!("assets/node_modules/highlight.js/styles/intellij-light.css"),
        "Ir Black"                 => include_bytes!("assets/node_modules/highlight.js/styles/ir-black.css"),
        "Isbl Editor Dark"         => include_bytes!("assets/node_modules/highlight.js/styles/isbl-editor-dark.css"),
        "Isbl Editor Light"        => include_bytes!("assets/node_modules/highlight.js/styles/isbl-editor-light.css"),
        "Kimbie Dark"              => include_bytes!("assets/node_modules/highlight.js/styles/kimbie-dark.css"),
        "Kimbie Light"             => include_bytes!("assets/node_modules/highlight.js/styles/kimbie-light.css"),
        "Lightfair"                => include_bytes!("assets/node_modules/highlight.js/styles/lightfair.css"),
        "Lioshi"                   => include_bytes!("assets/node_modules/highlight.js/styles/lioshi.css"),
        "Magula"                   => include_bytes!("assets/node_modules/highlight.js/styles/magula.css"),
        "Mono Blue"                => include_bytes!("assets/node_modules/highlight.js/styles/mono-blue.css"),
        "Monokai Sublime"          => include_bytes!("assets/node_modules/highlight.js/styles/monokai-sublime.css"),
        "Monokai"                  => include_bytes!("assets/node_modules/highlight.js/styles/monokai.css"),
        "Night Owl"                => include_bytes!("assets/node_modules/highlight.js/styles/night-owl.css"),
        "Nnfx Dark"                => include_bytes!("assets/node_modules/highlight.js/styles/nnfx-dark.css"),
        "Nnfx Light"               => include_bytes!("assets/node_modules/highlight.js/styles/nnfx-light.css"),
        "Nord"                     => include_bytes!("assets/node_modules/highlight.js/styles/nord.css"),
        "Obsidian"                 => include_bytes!("assets/node_modules/highlight.js/styles/obsidian.css"),
        "Panda Syntax Dark"        => include_bytes!("assets/node_modules/highlight.js/styles/panda-syntax-dark.css"),
        "Panda Syntax Light"       => include_bytes!("assets/node_modules/highlight.js/styles/panda-syntax-light.css"),
        "Paraiso Dark"             => include_bytes!("assets/node_modules/highlight.js/styles/paraiso-dark.css"),
        "Paraiso Light"            => include_bytes!("assets/node_modules/highlight.js/styles/paraiso-light.css"),
        "Pojoaque"                 => include_bytes!("assets/node_modules/highlight.js/styles/pojoaque.css"),
        "Purebasic"                => include_bytes!("assets/node_modules/highlight.js/styles/purebasic.css"),
        "Qtcreator Dark"           => include_bytes!("assets/node_modules/highlight.js/styles/qtcreator-dark.css"),
        "Qtcreator Light"          => include_bytes!("assets/node_modules/highlight.js/styles/qtcreator-light.css"),
        "Rainbow"                  => include_bytes!("assets/node_modules/highlight.js/styles/rainbow.css"),
        "Routeros"                 => include_bytes!("assets/node_modules/highlight.js/styles/routeros.css"),
        "School Book"              => include_bytes!("assets/node_modules/highlight.js/styles/school-book.css"),
        "Shades Of Purple"         => include_bytes!("assets/node_modules/highlight.js/styles/shades-of-purple.css"),
        "Srcery"                   => include_bytes!("assets/node_modules/highlight.js/styles/srcery.css"),
        "Stackoverflow Dark"       => include_bytes!("assets/node_modules/highlight.js/styles/stackoverflow-dark.css"),
        "Stackoverflow Light"      => include_bytes!("assets/node_modules/highlight.js/styles/stackoverflow-light.css"),
        "Sunburst"                 => include_bytes!("assets/node_modules/highlight.js/styles/sunburst.css"),
        "Tokyo Night Dark"         => include_bytes!("assets/node_modules/highlight.js/styles/tokyo-night-dark.css"),
        "Tokyo Night Light"        => include_bytes!("assets/node_modules/highlight.js/styles/tokyo-night-light.css"),
        "Tomorrow Night Blue"      => include_bytes!("assets/node_modules/highlight.js/styles/tomorrow-night-blue.css"),
        "Tomorrow Night Bright"    => include_bytes!("assets/node_modules/highlight.js/styles/tomorrow-night-bright.css"),
        "Vs"                       => include_bytes!("assets/node_modules/highlight.js/styles/vs.css"),
        "Vs2015"                   => include_bytes!("assets/node_modules/highlight.js/styles/vs2015.css"),
        "Xcode"                    => include_bytes!("assets/node_modules/highlight.js/styles/xcode.css"),
        "Xt256"                    => include_bytes!("assets/node_modules/highlight.js/styles/xt256.css"),
        name => {
            log::error!("Unknown highlight.js theme name {:?}. See https://highlightjs.org/static/demo/ to know the list", name);
            default
        }
    }
}

pub struct AssetsLoader {
    hljs_css: &'static [u8],
}

impl AssetsLoader {
    pub fn new(config: &Config, theme: WindowTheme) -> Self {
        let hl = config.highlight();
        let hljs_css = match theme {
            WindowTheme::Light => load_hljs_css(&hl.light, HLJS_DEFAULT_LIGHT_CSS),
            WindowTheme::Dark => load_hljs_css(&hl.dark, HLJS_DEFAULT_DARK_CSS),
        };
        Self { hljs_css }
    }

    pub fn load(&self, path: &str) -> (&'static [u8], &'static str) {
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
            "/"                    => INDEX_HTML,
            "/bundle.js"           => BUNDLE_JS,
            "/style.css"           => STYLE_CSS,
            "/github-markdown.css" => GITHUB_MARKDOWN_CSS,
            "/hljs-theme.css"      => self.hljs_css,
            "/tippy.css"           => TIPPY_CSS,
            "/tippy-light.css"     => TIPPY_LIGHT_CSS,
            "/logo.png"            => LOGO_PNG,
            _                      => unreachable!(),
        };

        (body, mime)
    }
}

#[derive(Default)]
pub struct Assets {
    loaded: [AtomicBool; ONETIME_ASSET_PATHS.len()],
}

impl Assets {
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
