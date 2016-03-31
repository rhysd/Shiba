Customization
=============

## Customize Shiba with YAML

You can put `config.yml` (__not__ `config.yaml`) in Shiba's application directory.  Application directory is `~/Library/Application\ Support/Shiba` for OS X, `~/.config/Shiba` for Linux, `%APPDATA%\Shiba` for Windows.
Below is an example for `config.yml`.

```YAML
width: 1000
height: "max"
linter: "remark-lint"
lint_options:
    maximum-line-length: false
drawer:
    responsive: false
menu:
    visible: false
markdown:
    font_size: 10px
```

All keys for the YAML configuration file is below:

| Key                   | Description                | Value                                           | Default                        |
| --------------------- | -------------------------- | ----------------------------------------------- | ------------------------------ |
| `width`               | Window width               | Number of pixel or `"max"`                      | `920`                          |
| `height`              | Window height              | Number of pixel or `"max"`                      | `800`                          |
| `linter`              | Linter name                | `"remark-lint"` or `"markdownlint"` or `"none"` | `"remark-lint"`                |
| `file_ext`            | Extensions to detect       | Array of extensions for each file types         | See below section              |
| `lint_options`        | Options for linter         | Depends on linter                               | Not specified                  |
| `shortcuts`           | Keyboard shortcuts         | Keyboard shortcuts definition                   | See below section              |
| `voice.enabled`       | Notify with voice          | enable/disable with boolean value               | false                          |
| `voice.source`        | Path to voice source       | Path string                                     | "../voices/bow.mp3"            |
| `drawer.responsive`   | Make drawer responsive     | Enable responsive drawer with boolean value     | true                           |
| `menu.visible`        | Left menu visibility       | Left menu is visible or not (boolean value)     | true                           |
| `ignore_path_pattern` | Regex to ignore  path      | Regex string which path should be ignored       | '[\\\\/]\\.' (dotfiles)        |
| `hide_title_bar`      | Hide a title bar (OS X)    | hide a tool bar if true                         | false                          |
| `markdown.font_size`  | Size of font in preview    | Specify font size by string (e.g. "10px")       | ''                             |
| `markdown.css_path`   | Path to css file to load   | Specify CSS file to style a markdown preview    | '/path/to/github-markdown.css' |
| `markdown.code_theme` | Color theme for code block | Specify highlight.js style theme for code block | 'github'                       |


## Customize Keyboard Shortcuts

You can customize the keyboard shortcuts as the value of `shortcuts` key in configuration.  You can specify a shortcut and corresponding action as key-value configuration.
Below is a vim-like keymaps customization example.
Shiba uses [Mousetrap](https://craig.is/killing/mice). Please see the Mousetrap's document to know how to write the key sequence.

```yaml
shortcuts:
    "g g": "PageTop"
    "shift+g": "PageBottom"
    i: ""
    m: ""
    command+q: "QuitApp"
    "shift+z shift+z": "QuitApp"
```

If an action is empty string `""` or `null`, the shortcut is disabled.


## CSS and Code Highlight

You can use your favorite CSS file and code highlighting theme for markdown preview.
Shiba uses [highlight.js](https://github.com/isagalaev/highlight.js), so you can specify the name of code theme supported by highlight.js.

`css_path` and `code_theme` entries are available to customize them.  Below is an example for using [tufte-css](https://github.com/edwardtufte/tufte-css) and ['Tomorrow' theme](https://highlightjs.org/static/demo/).

```yaml
markdown:
    css_path: '/path/to/tufte.css'
    code_theme: 'tomorrow'
```

![theme customization](https://raw.githubusercontent.com/rhysd/ss/master/Shiba/tufte-tomorrow.png)


## File Extensions

You can specify file extensions to watch with key `file_ext` as above table.
The extensions are array of string for each file types.  Below is default configuration.

```yaml
file_ext:
    markdown:
        - "md"
        - "markdown"
        - "mkd"
    html:
        - "html"
```

## User CSS

You can put `user.css` in configuration directory.  It is loaded at opening main window and enables to control the style of Shiba.  Below is an example to remove a reload button from menu.

```css
#reload-button {
  display: none;
}
```

Note that Shiba uses web components.  Some components' style can not be modified directly because Stylesheets are isolated in web components.  `Devtools` option may be useful to check the attributes in HTML document.

## Voice Notification

When linter reports some errors or warnings, Shiba can notify that with voice.
You can specify the resource for the voice as below.  You can use any format supported by `<audio>` tag.

```yaml
voice:
    enabled: true
    source: /path/to/your/favorite/sound
```


-----------------
[installation](installation.md) | [usage](usage.md) | [customization](customization.md) | [shortcuts](shortcuts.md) | [tips](tips.md)
