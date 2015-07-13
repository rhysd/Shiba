![shibainu](https://raw.githubusercontent.com/rhysd/Shiba/master/resource/image/doc-shibainu.png)
=====================

[![npm version](https://img.shields.io/npm/v/shiba.svg?style=flat-square)](https://www.npmjs.com/package/shiba)

Shiba is a rich live markdown preview app with lint.  It watches markdown files in specific directory and automatically shows the preview and result of lint.
Shiba is built on [Electron](https://github.com/atom/electron) and [Polymer](https://www.polymer-project.org/1.0/).

- Rich GFM (code highlight, emoji)
- Live reload
- Automatic lint (mdast, markdownlint)
- Easy to install
- Customizable (yaml configuration file)
- Cross platform (Mac, Linux, Windows)
- Dog respected :dog2:

## Installation

### General

Experimental alpha release is available for Linux, OS X and Windows.
Please download from [here](https://github.com/rhysd/Shiba/releases) and unzip the archive.

- __Linux__ : Use `shiba` executable in the directory.
- __OS X__ : Use `Shiba.app` in the directory (e.g. `$ open -a ./Shiba.app`).
- __Windows__ : Use `shiba.exe` in the directory.  No need to install.

### Via npm

__NOW BROKEN.__

```
$ npm install -g shiba
```

### For development

```sh
$ git clone https://github.com/rhysd/Shiba.git && cd Shiba
$ bower install && npm install
$ electron . # Or `electron . {file to watch}`
```

## Usage

![Shiba on Linux](https://raw.githubusercontent.com/rhysd/screenshots/master/Shiba/shiba-main-0.1.0.png)

- When file is updated, Shiba automatically updates preview and lint result.
- You can see the result of lint by pushing lint icon in right above.  If linter reports any error, the icon's color is changed to red.
- You can change the watching file by pushing directory icon in right above.
- You can quit the app by closing window (`Command+Q` shortcut is also available in OS X).

### Watch specific file

```sh
# Linux
$ shiba /path/to/markdown-file

# OS X
$ open -a Shiba.app /path/to/markdown-file
# or
$ Shiba.app/Contents/MacOS/Shiba /path/to/markdown-file

# Windows
$ shiba.exe /path/to/markdown-file
```

Please specify the markdown file you want to watch as an argument of command.

### Watch files in specific directory

```sh
# Linux
$ shiba /path/to/dir

# OS X
$ open -a Shiba.app /path/to/dir

# Windows
$ shiba.exe /path/to/dir
```

Instead of markdown file, please specify the path to directory as above.  If you omit an argument, current working directory would be watched.


## Customization

You can put `config.yml` (__not__ `config.yaml`) in Shiba's application directory.  Application directory is `~/Library/Application\ Support/Shiba` for OS X, `~/.config/Shiba` for Linux.
Below is an example for `config.yml`.

```YAML
width: 800
height: "max"
linter: "mdast-lint"
lint_options:
    maximum-line-length: false
```

| Key            | Description        | Value                                        | Default                     |
| -------------- | ------------------ | -------------------------------------------- | --------------------------- |
| `width`        | Window width       | Number of pixel or `"max"`                   | `800`                       |
| `height`       | Window height      | Number of pixel or `"max"`                   | `600`                       |
| `linter`       | Linter name        | `"mdast-lint"` or `"markdownlint"` or "none" | `"mdast-lint"`              |
| `file_ext`     | Ext to detect      | Array of extensions                          | `["md", "markdown", "mkd"]` |
| `lint_options` | Options for linter | Depends on linter                            | Not specified               |

## TODOs

- [ ] Keyboard shortcut
- [ ] Package installer if needed (Windows, OS X)
- [ ] Use file(directory?) picker to set watching path
- Smarter alternatives
  - [ ] [slim](https://github.com/slim-template/slim)
  - [ ] [TypeScript](http://www.typescriptlang.org/)
  - [ ] [sass](http://sass-lang.com/)

## Known Issues

- URL links in document
- Image path
- Japanese is shown as tofu (font issue) in Linux

## Special Thanks

- The logo of this app came from [いらすとや](http://www.irasutoya.com/).
- This app was inspired by [@mattn](https://github.com/mattn)'s [mkup](https://github.com/mattn/mkup).
- This app refers [vmd](https://github.com/yoshuawuyts/vmd) a lot, which is a very simple markdown preview app built on Electron.
- Emoji pictures are from [arvida/emoji-cheat-sheet.com](https://github.com/arvida/emoji-cheat-sheet.com).

## License

MIT License.

    Copyright (c) 2015 rhysd

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
    of the Software, and to permit persons to whom the Software is furnished to do so,
    subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
    INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
    PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
    LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
    TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR
    THE USE OR OTHER DEALINGS IN THE SOFTWARE.

