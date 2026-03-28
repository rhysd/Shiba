<a id="v2.0.0-alpha.4"></a>
# [v2.0.0-alpha.4](https://github.com/rhysd/Shiba/releases/tag/v2.0.0-alpha.4) - 2026-03-28

- Support multiple windows so that multiple documents can be previewed at the same time. Now a single Shiba process can open multiple windows. When the last window is closed, or 'Quit' menu item or key action is selected, the process exits.
  - Implement the following ways to open new windows in several situations.
    - Clicking a link to local markdown file with pressing <kbd>Shift</kbd> key opens the file in a new window.
    - Clicking a hash link (e.g. `[link](#hash)`) with pressing <kbd>Shift</kbd> key opens the current document in a new window and automatically scrolling to the linked element.
    - 'New Window' menu item or `NewWindow` key action opens a new empty window.
    - 'Duplicate Window' menu item or `DuplicateWindow` key action duplicates the current window.
    - 'Open in New Window' menu item or `OpenFileInNewWindow` key action picks files in a file dialog and opens them in respective new windows.
    - In the history palette, hit <kbd>Enter</kbd> or click a history item with pressing <kbd>Shift</kbd> to open the selected item in a new window.
    - In the outline palette, hit <kbd>Enter</kbd> or click a section with pressing <kbd>Shift</kbd> to open the current document in a new window and automatically scroll to the selected section.
    - Click a section in the side bar to open the current document in a new window and automatically scroll to the selected section.
    - `--open` (or `-o`) option in the command line arguments opens a given path with a new window. See `--help` output for more details.
  - Add the following menu items and key actions related to multiple windows.
    - 'Close Window' menu item and `CloseWindow` key action to close the current window.
    - 'Close All Other Windows' menu item and `CloseAllOtherWindows` key action to close all windows except the current one.
    - 'Bring all to front' window menu item on macOS.
- Return non-zero exit status when at least one unexpected error was caused.
- Fix notifications are included in a printed pages.
- Update wry to 0.55 and tao to 0.35.
- Don't set 'pre-release' on GitHub release page so that the latest alpha release is linked from the repository page.

[Changes][v2.0.0-alpha.4]


<a id="v2.0.0-alpha.3"></a>
# [v2.0.0-alpha.3](https://github.com/rhysd/Shiba/releases/tag/v2.0.0-alpha.3) - 2026-03-11

- **Breaking change:** Rename key actions `Forward` and `Back` to more clear names `GoForward` and `GoBack`.
- **Breaking chagne:** There were two key shortcut systems in Shiba; (1) our own key shortcut system configurable in `config.yml` and (2) platform-specific menu items. However two systems existing in one app was confusing and the behavior of menu item key shortcuts depended on the platforms. This release removes platform-specific key shortcuts in menu items and unifies all key actions into our own key shortcut system which is configurable at the `keymaps` section of the configuration file. The following key actions are newly introduced for this change.
  - `MaximizeWindow`
  - `MinimizeWindow`
  - `ToggleAlwaysOnTop`
  - `ToggleMenuBar`
  - `ShowMenu`
  - `EditConfig`
- **Breaking change:** Following the above change, the default key mappings are overhauled. The new mappings are as follows. If you want to update your existing configuration to the new mappings, overwrite it by `--generate-config-file` command line option.
  | Keys | Action |
  |-|-|
  | `j` | `ScrollDown` |
  | `k` | `ScrollUp` |
  | `h` | `ScrollLeft` |
  | `l` | `ScrollRight` |
  | `g` | `ScrollTop` |
  | `G` | `ScrollBottom` |
  | `d` | `ScrollPageDown` |
  | `u` | `ScrollPageUp` |
  | `space` | `ScrollPageDown` |
  | `down` | `ScrollDown` |
  | `up` | `ScrollUp` |
  | `left` | `ScrollLeft` |
  | `right` | `ScrollRight` |
  | `pagedown` | `ScrollPageDown` |
  | `pageup` | `ScrollPageUp` |
  | `ctrl+down` | `ScrollPageDown` |
  | `ctrl+up` | `ScrollPageUp` |
  | `ctrl+shift+down` | `ScrollBottom` |
  | `ctrl+shift+up` | `ScrollTop` |
  | `ctrl+j` | `ScrollNextSection` |
  | `ctrl+k` | `ScrollPrevSection` |
  | `ctrl+b` | `GoBack` |
  | `ctrl+f` | `GoForward` |
  | `ctrl+o` | `OpenFile` |
  | `ctrl+shift+o` | `OpenDir` |
  | `ctrl+h` | `History` |
  | `ctrl+r` | `Reload` |
  | `o` | `Outline` |
  | `s` | `Search` |
  | `plus` | `ZoomIn` |
  | `-` | `ZoomOut` |
  | `ctrl+m` | `MaximizeWindow` |
  | `mod+q` | `Quit` |
  | `?` | `Help` |
- Apply the [Mica material](https://learn.microsoft.com/en-us/windows/apps/design/style/mica) to the window background on Windows 11 or later, which is the modern dynamic material based on your desktop wallpaper.
  - <img alt="window with mica material" src="https://github.com/user-attachments/assets/cfbb2b87-886b-4770-ac24-467047ca89e6" width="598">
- Support `max` keyword at the width and hight of window size in the `window.default_size` configuration. For example the following configration creates a window with fixed 800 pixels width and maximized height.
  ```yaml
  window:
    default_size:
      width: 800
      height: max
  ```
- Add new `window.vibrant` configuration. When this is set to `true`, Shiba applies platform-specific vibrant effects to the window; [`NSVisualEffectView` on macOS](https://developer.apple.com/design/human-interface-guidelines/materials) and [Mica material](https://learn.microsoft.com/en-us/windows/apps/design/style/mica) on Windows, no effect on Linux. Setting `false` disables the dynamic effects by using solid colors. It may slightly improve the performance of the application rendering and launch. The default value of this configuration is `true`.
  ```yaml
  window:
    vibrant: true  # Apply vibrant effect to the window
  ```
- Set the default window size to 600x800.
- Add 'Delete History' menu item in the 'History' menu to delete the history.
- Add `GoTop` key action and a new menu item to go to the top of the history. It quickly opens the most recent document in the history.
- Add `no-debug-log` cargo feature to disable the debug log statically. It can slightly reduce the binary size and application performance.
- Fix 'Print' menu item only prints the current window. Now it prints all pages and doesn't include the side bar.
- Fix applying the system's theme setting to the webview on Windows.
- Implement platform-agnostic window maximization/minimization so that 'Maximize Window' and 'Minimize Window' menu items are available on all platforms.
- Efficiently receive menu item events instead of polling them at every window events.
- Update npm dependencies including some security fixes.

[Changes][v2.0.0-alpha.3]


<a id="v2.0.0-alpha.2"></a>
# [v2.0.0-alpha.2](https://github.com/rhysd/Shiba/releases/tag/v2.0.0-alpha.2) - 2026-02-21

- **BREAKING CHANGE**: Rename `preview.recent_files` setting to `preview.history_size`. This needs fix in the configuration file. Please fix it manually or re-generate it by the `--generate-config-file` command line option.
  ```diff
   preview:
  -  recent_files: 100
  +  history_size: 100
  ```
- Move the reopened history item to the top of the history so that it can be easily accessed later.
- Set the application icon in dock when Shiba is run from terminal on macOS.
- Avoid a white screen flicker when opening the application with a large Markdown file in dark mode.
- Remove file paths which don't exist while navigating the history with `Forward`/`Back`.
- Set the parent window to the dialogs.
- Let OS determine the default current directory of file dialogs unless `dialog.default_dir` setting is specified. Note that Shiba launched from terminal still prioritizes the terminal's current working directory on macOS.
- Shiba is now released on [crates.io](https://crates.io/crates/shiba-preview). Shiba can be installed via `cargo` command.
  ```sh
  cargo install shiba-preview@2.0.0-alpha.2
  ```
- Add categories and keywords to Cargo.toml.
- Fix webview is not rendered on Linux because the webview panel is inserted to an incorrect box.
- Avoid app crash when creating two Shiba processes on Linux by removing the application ID.
- Fix the latest history item is skipped when navigating with `Back` from the welcome page.
- Fix titles of file dialogs.
- Update cargo dependencies including wry v0.54 and the security fix for `time` crate.
- Update npm dependencies including mermaid security fix.

[Changes][v2.0.0-alpha.2]


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


[v2.0.0-alpha.4]: https://github.com/rhysd/Shiba/compare/v2.0.0-alpha.3...v2.0.0-alpha.4
[v2.0.0-alpha.3]: https://github.com/rhysd/Shiba/compare/v2.0.0-alpha.2...v2.0.0-alpha.3
[v2.0.0-alpha.2]: https://github.com/rhysd/Shiba/compare/v2.0.0-alpha.1...v2.0.0-alpha.2
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
