Customization
=============

## Customize Shiba with YAML

You can put `config.yml` (__not__ `config.yaml`) in Shiba's application directory.  Application directory is `~/Library/Application\ Support/Shiba` for OS X, `~/.config/Shiba` for Linux.
Below is an example for `config.yml`.

```YAML
width: 800
height: "max"
linter: "mdast-lint"
lint_options:
    maximum-line-length: false
```

All keys for the YAML configuration file is below:

| Key             | Description          | Value                                        | Default             |
| --------------- | -------------------- | -------------------------------------------- | ------------------- |
| `width`         | Window width         | Number of pixel or `"max"`                   | `800`               |
| `height`        | Window height        | Number of pixel or `"max"`                   | `600`               |
| `linter`        | Linter name          | `"mdast-lint"` or `"markdownlint"` or "none" | `"mdast-lint"`      |
| `file_ext`      | Extensions to detect | Array of extensions for each file types      | See below section   |
| `lint_options`  | Options for linter   | Depends on linter                            | Not specified       |
| `shortcuts`     | Keyboard shortcuts   | Keyboard shortcuts definition                | See below section   |
| `voice.enabled` | Notify with voice    | enable/disable with boolean value            | false               |
| `voice.source`  | Path to voice source | Path string                                  | "../voices/bow.mp3" |


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

## Voice Notification

When linter reports some errors or warnings, Shiba can notify that with voice.
You can specify the resource for the voice as below.  You can use any format supported by `<audio>` tag.

```yaml
voice:
    enabled: true
    source: /path/to/your/favorite/sound
```


