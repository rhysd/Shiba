use crate::config::Config;
use crate::renderer::Theme as WindowTheme;
use phf::phf_map;
use std::borrow::Cow;
use std::fs;

#[cfg(debug_assertions)]
const BUNDLE_JS: &[u8] = include_bytes!("assets/bundle.js");
#[cfg(debug_assertions)]
const BUNDLE_JS_MAP: &[u8] = include_bytes!("assets/bundle.js.map");
#[cfg(not(debug_assertions))]
const BUNDLE_JS: &[u8] = include_bytes!("assets/bundle.min.js");
const INDEX_HTML: &[u8] = include_bytes!("assets/index.html");
const GITHUB_MARKDOWN_CSS: &[u8] =
    include_bytes!("assets/node_modules/github-markdown-css/github-markdown.css");
const STYLE_CSS: &[u8] = include_bytes!("assets/web/style.css");
const HLJS_DEFAULT_LIGHT_CSS: &[u8] =
    include_bytes!("assets/node_modules/highlight.js/styles/github.css");
const HLJS_DEFAULT_DARK_CSS: &[u8] =
    include_bytes!("assets/node_modules/highlight.js/styles/github-dark.css");
const HLJS_DIMMED_DARK_CSS: &[u8] =
    include_bytes!("assets/node_modules/highlight.js/styles/github-dark-dimmed.css");
const LOGO_PNG: &[u8] = include_bytes!("assets/logo.png");

#[rustfmt::skip]
const HLJS_CSS_TABLE: phf::Map<&'static str, &'static [u8]> = phf_map! {
    "GitHub"                   => HLJS_DEFAULT_LIGHT_CSS,
    "Github"                   => HLJS_DEFAULT_LIGHT_CSS,
    "GitHub Dark"              => HLJS_DEFAULT_DARK_CSS,
    "Github Dark"              => HLJS_DEFAULT_DARK_CSS,
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
    "Github Dark Dimmed"       => HLJS_DIMMED_DARK_CSS,
    "GitHub Dark Dimmed"       => HLJS_DIMMED_DARK_CSS,
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
};

// https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types
// https://www.iana.org/assignments/media-types/media-types.xhtml
#[rustfmt::skip]
const MIME_TABLE: phf::Map<&'static str, &'static str> = phf_map! {
    "html" => "text/html;charset=UTF-8",
    "js"   => "text/javascript;charset=UTF-8",
    "css"  => "text/css;charset=UTF-8",
    "apng" => "image/apng",
    "avif" => "image/avif",
    "bmp"  => "image/bmp",
    "gif"  => "image/gif",
    "jpg"  => "image/jpeg",
    "jpeg" => "image/jpeg",
    "png"  => "image/png",
    "svg"  => "image/svg+xml",
    "webp" => "image/webp",
    "tiff" => "image/tiff",
    "map"  => "text/plain;charset=UTF-8",
};

fn load_hljs_css(theme_name: &str, default: &'static [u8]) -> &'static [u8] {
    log::debug!("Loading highlight.js theme {:?}", theme_name);
    if let Some(css) = HLJS_CSS_TABLE.get(theme_name) {
        css
    } else {
        log::error!("Unknown highlight.js theme name {:?}. See https://highlightjs.org/static/demo/ to know the list", theme_name);
        default
    }
}

fn load_user_css(config: &Config) -> Option<Vec<u8>> {
    let config_path = config.config_file()?;
    let css_path = config.preview().css_path()?;
    let css_path = config_path.parent()?.join(css_path);

    log::debug!("Loading user CSS at {:?}", css_path);
    match fs::read(&css_path) {
        Ok(css) => Some(css),
        Err(err) => {
            log::error!(
                "Could not load CSS file {:?} specified in config file {:?}: {}",
                css_path,
                config_path,
                err,
            );
            None
        }
    }
}

fn guess_mime(path: &str) -> &'static str {
    if let Some(idx) = path.rfind('.') {
        if let Some(mime) = MIME_TABLE.get(&path[idx + 1..]) {
            return mime;
        }
    }
    log::error!("Unknown mime type for {:?}. Fallback to octet-stream", path);
    "application/octet-stream"
}

pub struct Assets {
    hljs_css: &'static [u8],
    markdown_css: Cow<'static, [u8]>,
}

impl Assets {
    pub fn new(config: &Config, theme: WindowTheme) -> Self {
        let hl = config.preview().highlight();
        let hljs_css = match theme {
            WindowTheme::Light => load_hljs_css(&hl.light, HLJS_DEFAULT_LIGHT_CSS),
            WindowTheme::Dark => load_hljs_css(&hl.dark, HLJS_DEFAULT_DARK_CSS),
        };

        let markdown_css = if let Some(css) = load_user_css(config) {
            Cow::Owned(css)
        } else {
            Cow::Borrowed(GITHUB_MARKDOWN_CSS)
        };

        Self { hljs_css, markdown_css }
    }

    pub fn load(&self, path: &str) -> (Vec<u8>, &'static str) {
        let mime = guess_mime(path);

        #[rustfmt::skip]
        let body = match path {
            "/index.html"          => INDEX_HTML.into(),
            "/bundle.js"           => BUNDLE_JS.into(),
            "/style.css"           => STYLE_CSS.into(),
            "/github-markdown.css" => self.markdown_css.to_vec(),
            "/hljs-theme.css"      => self.hljs_css.into(),
            "/logo.png"            => LOGO_PNG.into(),
            #[cfg(debug_assertions)]
            "/bundle.js.map"       => BUNDLE_JS_MAP.into(),
            path                   => {
                log::debug!("Dynamically loading external resource {:?}", path);
                match fs::read(path) {
                    Ok(content) => content,
                    Err(err) => {
                        log::error!("Could not read external resource {:?}: {}", path, err);
                        vec![]
                    }
                }
            }
        };

        (body, mime)
    }
}
