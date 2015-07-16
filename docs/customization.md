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

| Key            | Description          | Value                                        | Default                     |
| -------------- | -------------------- | -------------------------------------------- | --------------------------- |
| `width`        | Window width         | Number of pixel or `"max"`                   | `800`                       |
| `height`       | Window height        | Number of pixel or `"max"`                   | `600`                       |
| `linter`       | Linter name          | `"mdast-lint"` or `"markdownlint"` or "none" | `"mdast-lint"`              |
| `file_ext`     | Extensions to detect | Array of extensions                          | `["md", "markdown", "mkd"]` |
| `lint_options` | Options for linter   | Depends on linter                            | Not specified               |
| `shortcuts`    | Keyboard shortcuts   | Keyboard shortcuts definition                | See below section           |


## Customize Keyboard Shortcuts

You can customize the keyboard shortcuts as the value of `shortcuts` key in configuration.  You can specify a shortcut and corresponding action as key-value configuration.
Customization example for `config.yml` is below.  Typing `O`, `K`, `L`, `,` are mapping to actions `Up`, `Left`, `Right`, `Down`. Typing `Control` (or `Command` for OS X) key and `s` key executes `Lint` action.  And `J` key is disabled to avoid mistyping.

The format of key shortcuts is the same as [Accelerator](https://github.com/atom/electron/blob/master/docs/api/accelerator.md) in Electron.

```yaml
shortcuts:
    O: "Up"
    K: "Left"
    L: "Right"
    ,: "Down"
    J: ""
    CommandOrControl+S: "Lint"
```

If an action is empty string `""` or `null`, the shortcut is disabled.

