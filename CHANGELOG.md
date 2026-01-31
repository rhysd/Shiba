<a id="v2.0.0-alpha.1"></a>
# [v2.0.0-alpha.1](https://github.com/rhysd/Shiba/releases/tag/v2.0.0-alpha.1) - 2026-01-31

- Compress the bundled JavaScript source with [Zstandard](https://facebook.github.io/zstd/) algorithm for release builds. This made the binary size 1.45x smaller.
- Allow selecting multiple files and directories at once via dialogs.
- Upgrade MathJax to [v4](https://docs.mathjax.org/en/latest/upgrading/whats-new-4.0.html), which is the culmination of several years of work.
- Support [physics notations](https://docs.mathjax.org/en/latest/input/tex/extensions/physics.html) in math expressions.
- History implementation was refactored with [index map](https://docs.rs/indexmap/latest/indexmap/) data structure.
- Migrate from commonjs to ESM. This made the bundle size about 6% smaller.
- Change the environment variables for debug logs
  - `RUST_LOG` → `SHIBA_LOG`
  - `RUST_LOG_STYLE` → `SHIBA_LOG_STYLE`
- Fix the saved history items can be truncated.
- Fix file paths passed via command line arguments are not registered to the history
- Fix watching the same path multiple times.
- Fix error messages output to a terminal are discarded in release builds on Windows.
- Fix app crashes when the document contains U+0000 inside math expressions due to MathJax parser.
- Update cargo dependencies including rfd v0.17
- Update npm dependencies

[Changes][v2.0.0-alpha.1]


<a id="v2.0.0-alpha.0"></a>
# [v2.0.0-alpha.0](https://github.com/rhysd/Shiba/releases/tag/v2.0.0-alpha.0) - 2026-01-31

This is the first experimental release of Shiba v2. Shiba v2 is the complete rewrite of v1 using Rust and platform-specific WebView. Please read the [README.md file](https://github.com/rhysd/Shiba?tab=readme-ov-file) for more details.

> [!WARNING]
> Shiba v2 is a work in progress


For v1, please go to the [`v1` branch](https://github.com/rhysd/Shiba/tree/v1).


[Changes][v2.0.0-alpha.0]


<a id="v1.2.1"></a>
# [Version 1.2.1 (v1.2.1)](https://github.com/rhysd/Shiba/releases/tag/v1.2.1) - 2018-05-28

- Update dependencies
  - Use the latest Electron v2.0.2
  - Build with the newest TypeScript compiler
  - Some other packages are updated


[Changes][v1.2.1]


<a id="v1.2.0"></a>
# [Version 1.2.0 (v1.2.0)](https://github.com/rhysd/Shiba/releases/tag/v1.2.0) - 2018-03-17

- Improved sanitizatioin
  - All HTML elements were banned in v1.1.1, but it was too strict
  - As of GitHub, now Shiba allows some non-harmful HTML elements in documents (e.g. `<a name="..."></a>`
  - Please see [the full list](https://github.com/rhysd/marked-sanitizer-github#sanitized-elements) to know the details
- Update dependencies (Electron v1.8.4)

[Changes][v1.2.0]


<a id="v1.1.2"></a>
# [Version 1.1.2 (v1.1.2)](https://github.com/rhysd/Shiba/releases/tag/v1.1.2) - 2018-02-01

- Update dependencies
  - Including security bugfix for Electron CVE-2018-1000006

[Changes][v1.1.2]


<a id="v1.1.1"></a>
# [Version 1.1.1 (v1.1.1)](https://github.com/rhysd/Shiba/releases/tag/v1.1.1) - 2017-11-28

- Add new 367 emojis 👯 
- Enable to choose rules in [remark-lint](https://github.com/wooorm/remark-lint) config
- Fix sanitize issue of marked parser
- Update dependencies





[Changes][v1.1.1]


<a id="v1.1.0"></a>
# [Version 1.1.0 (v1.1.0)](https://github.com/rhysd/Shiba/releases/tag/v1.1.0) - 2017-11-03

- Large dependencies updates. All packages are up to date
  - Electron v1.7.9
  - Polymer v1.11
  - ...
- Also accept `config.yaml` as well as `config.yml`
- Improve CLI options and handling arguments
- Stop following symlinks by default because it may cause performance issue on a directly containing so many files and directories
- `follow_symlinks` and `default_path` config option are added
- Show window after app contents are loaded in order to avoid whole white screen
- Now markdown linter checks only consistency by default
- Fix links on Windows ([#37](https://github.com/rhysd/Shiba/issues/37))
- Fix links which contain images ([#38](https://github.com/rhysd/Shiba/issues/38))
- Fix watching path is broken when unsupported kind of file is D&Ded



[Changes][v1.1.0]


<a id="v1.0.4"></a>
# [Version 1.0.4 (v1.0.4)](https://github.com/rhysd/Shiba/releases/tag/v1.0.4) - 2016-11-29

This is minor release for tiny improvement and fix.
- Restore last window state on start. And add `restore_window_state` configuration to enable/disable it
- Convert non-UTF8 encoded documents if needed
- Update Electron binary to v.1.4.10 (including security fix)
- Fix Japanese problems on search box
- Fix scrolling issue on Windows (10?) (working in progress)
- Fix emoji parsing problem


[Changes][v1.0.4]


<a id="v1.0.0"></a>
# [v1.0.0 Release](https://github.com/rhysd/Shiba/releases/tag/v1.0.0) - 2016-04-06

First major release :100: :dog:

### New Features
- [x] task list in markdown
- Add tooltips to links
- Search text in the document
- Outline window
- User CSS and favorite code theme
- Math formula preview with [katex](https://github.com/Khan/KaTeX) (please use `katex` code block)
- [mermaid.js](https://github.com/ludwick/reveal.js-mermaid-plugin) integration (please use `mermaid` code block)
- Hidden title bar on OS X
- Many more configurations

### Improvements
- Move menu from right to left because document is align to left.  Minimize move of eyes.
- Improve rendering performance (2x faster)
- Optimize app start up time (mermaid, lint result window setup, and so on)
- Update Electron to 0.37
- Use native dialog to choose a file or directory
- Add more tests
- Many refactorings
- Easily install with `brew cask install shiba` on OS X

### Fixes
- Replace :emoji: only in text
- Fix document layout for print
- Fix many bugs

### Documents
- [README](https://github.com/rhysd/Shiba/tree/master/README.md)
- [Installation](https://github.com/rhysd/Shiba/blob/master/docs/installation.md)
- [Usage](https://github.com/rhysd/Shiba/blob/master/docs/usage.md)
- [Customization](https://github.com/rhysd/Shiba/blob/master/docs/customization.md)
- [Shortcuts](https://github.com/rhysd/Shiba/blob/master/docs/shortcuts.md)
- [Tips](https://github.com/rhysd/Shiba/blob/master/docs/tips.md)

---

Confirmed on
- OS X 10.11.4
- Ubuntu 15.10
- Windows 8.1


[Changes][v1.0.0]


<a id="0.4.0"></a>
# [Version 0.4.0](https://github.com/rhysd/Shiba/releases/tag/0.4.0) - 2015-08-12

Many improvements and fixes since version 0.3.4!

Installing Shiba in each OS is very easy.  Please see [installation document](https://github.com/rhysd/Shiba/blob/master/docs/installation.md).

### Changes
- **Improvements**
  - Open local markdown links with preview
  - Use Meiryo font on Windows to improve Japanese document
  - Add drop zone in main window at start up
  - Make main window always file-droppable
  - Add `Reload` action and default key shortcut
  - `drawer.responsive` and `menu.visible` configuration options
  - `{config dir}/user.css`
- **Fixes**
  - Ensure to make lint messages empty before executing linter
  - Fix opening external links
  - Fix `#hash` local links
  - Fix links which contain white spaces

Thank you [@xHN35RQ](https://github.com/xHN35RQ) so much!


[Changes][0.4.0]


<a id="v0.3.4"></a>
# [Version 0.3.4 (v0.3.4)](https://github.com/rhysd/Shiba/releases/tag/v0.3.4) - 2015-07-31

How to install is [here](https://github.com/rhysd/Shiba/blob/master/docs/installation.md)

## Changes
- HTML preview
- handling keyshortcuts in renderer
- voice notification
- `--detach` option for CLI
- reduce dependency
- open external page links with external browser


[Changes][v0.3.4]


<a id="v0.3.0"></a>
# [v0.3.0](https://github.com/rhysd/Shiba/releases/tag/v0.3.0) - 2015-07-26

Note that the file is a bit large because it includes chromium to execute Electron app.

### Linux

Download and unzip the archive file.
You can find executable `shiba` in the directory.  Please simply execute it.

### OS X

Download and unzip the archive file.
You can find `Shiba.app` in the directory.
Please simply execute `open -a Shiba` to execute Shiba from command line.  Note that `open` command can't maintain the command line arguments.  If you want to pass command line arguments, please use executable `Shiba.app/Contents/MacOS/Shiba` directly.

If you want to put Shiba.app in Dock and start with double click, put `Shiba.app` to your Application directory (`~/Applications` or `/Applications`).

### Windows

Download and unzip the archive file.
You can find `shiba.exe` in the directory.  Please simply use it by double click or from command prompt.  You need to install nothing.


[Changes][v0.3.0]


<a id="v0.1.0"></a>
# [v0.1.0](https://github.com/rhysd/Shiba/releases/tag/v0.1.0) - 2015-07-13

First experimental alpha release of Shiba.
- Linux   : Use `shiba` executable in the directory.
- OS X    : Use `Shiba.app` in the directory (e.g. `$ open -a ./Shiba.app`).
- Windows : Use `shiba.exe` in the directory.  No need to install.


[Changes][v0.1.0]


[v2.0.0-alpha.1]: https://github.com/rhysd/Shiba/compare/v2.0.0-alpha.0...v2.0.0-alpha.1
[v2.0.0-alpha.0]: https://github.com/rhysd/Shiba/compare/v1.2.1...v2.0.0-alpha.0
[v1.2.1]: https://github.com/rhysd/Shiba/compare/v1.2.0...v1.2.1
[v1.2.0]: https://github.com/rhysd/Shiba/compare/v1.1.2...v1.2.0
[v1.1.2]: https://github.com/rhysd/Shiba/compare/v1.1.1...v1.1.2
[v1.1.1]: https://github.com/rhysd/Shiba/compare/v1.1.0...v1.1.1
[v1.1.0]: https://github.com/rhysd/Shiba/compare/v1.0.4...v1.1.0
[v1.0.4]: https://github.com/rhysd/Shiba/compare/v1.0.0...v1.0.4
[v1.0.0]: https://github.com/rhysd/Shiba/compare/0.4.0...v1.0.0
[0.4.0]: https://github.com/rhysd/Shiba/compare/v0.3.4...0.4.0
[v0.3.4]: https://github.com/rhysd/Shiba/compare/v0.3.0...v0.3.4
[v0.3.0]: https://github.com/rhysd/Shiba/compare/v0.1.0...v0.3.0
[v0.1.0]: https://github.com/rhysd/Shiba/tree/v0.1.0

<!-- Generated by https://github.com/rhysd/changelog-from-release v3.9.1 -->
