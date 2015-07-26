![shibainu](https://raw.githubusercontent.com/rhysd/Shiba/master/resource/image/doc-shibainu.png)
=====================

[![npm version](https://img.shields.io/npm/v/shiba.svg?style=flat-square)](https://www.npmjs.com/package/shiba)
[![Build Status](https://travis-ci.org/rhysd/Shiba.svg)](https://travis-ci.org/rhysd/Shiba)

Shiba is a rich live markdown preview app with lint.  It watches markdown files in specific directory and automatically shows the preview and result of lint.
Shiba is built on [Electron](https://github.com/atom/electron) and [Polymer](https://www.polymer-project.org/1.0/).

- Rich GFM (code highlight, emoji)
- Live reload
- Automatic lint (mdast-lint, markdownlint)
- Cross platform (Mac, Linux, Windows)
- [Easy to install](docs/installation.md)
- [Keyboard shortcuts](docs/shortcuts.md)
- [Customizable with YAML config file](docs/customization.md) (keyboard shortcuts, linter, etc)
- HTML preview
- Dog-respected :dog2:

## Installation

You can install Shiba easily.  Please see [installation document](docs/installation.md).

## Usage

![Shiba on Linux](https://raw.githubusercontent.com/rhysd/screenshots/master/Shiba/shiba-main-0.1.0.png)

1. At Shiba starting up, it is watching the current working directory (watching directory is shown in title of window).
2. When you edit the markdown file in current working directory, shiba finds the update, renders the file in window and set the result of lint.
3. You can see the result of lint by pushing the '!' button in right above of window.  When the button is red, it means that linter reported some errors.
4. You can change the watching directory/file using 'directory' button in right above of window.  If you choose a file, Shiba watches the file only.  If you choose a directory, Shiba watches all files in the directory.  Wathing path is shown in title of window.
5. You can quit app by closing the window.

Please see [usage document](docs/usage.md) for more detail.

## Keyboard Shortcuts

Keyboard shortcuts are available for above all operations.
Please refer [shortcuts document](docs/shortcuts.md).


## Customization

You can customize Shiba by making YAML configuration file.
Please refer [customization document](docs/customization.md).


## TODOs

- Outline
- Search
- More formats
  - [ ] HTML with `<webview>` tag (WIP)
  - [ ] PDF with [PDF.js](https://mozilla.github.io/pdf.js/)
- [ ] Tests :scream:


## Known Issues

- URL links to local document
- Japanese may be shown as tofu (font issue) in Linux


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

