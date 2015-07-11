![shibainu](https://raw.githubusercontent.com/rhysd/Shiba/master/resource/image/doc-shibainu.png)
=====================

[![npm version](https://img.shields.io/npm/v/shiba.svg?style=flat-square)](https://www.npmjs.com/package/shiba)

Shiba is a rich live markdown preview app with lint.  It watches markdown files in specific directory and automatically shows the preview and result of lint.
Shiba is built on [Electron](https://github.com/atom/electron) and [Polymer](https://www.polymer-project.org/1.0/).

- Rich GFM (code highlight, emoji)
- Live reload
- Automatic lint (mdast, markdownlint)
- One executable
- Customizable (yaml configuration file)
- Cross platform (Mac, Linux, Windows)
- Dog respected :dog2:

## Installation

### General

Not released yet.

### Via npm

```
$ npm install -g shiba
```

### For development

```sh
$ git clone https://github.com/rhysd/Shiba.git && cd Shiba
$ bower install && npm install
$ electron . # Or `electron . {file to watch}`
```

When the first version is released, all you have to do will be only downloading one file and place it.

## Usage

T.B.D

## TODOs

- [ ] Prettier lint
- [ ] Keyboard shortcut
- [ ] Configuration with YAML
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

