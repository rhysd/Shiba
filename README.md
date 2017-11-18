![shibainu](https://raw.githubusercontent.com/rhysd/ss/master/Shiba/title-bar.jpg)
=====================

[![npm version](https://img.shields.io/npm/v/shiba.svg?style=flat-square)](https://www.npmjs.com/package/shiba)
[![Build Status](https://travis-ci.org/rhysd/Shiba.svg?branch=master)](https://travis-ci.org/rhysd/Shiba)

Shiba is a rich live markdown preview app with linter.  It watches markdown files in specific directory and automatically shows the preview and result of lint.
Shiba is built on [Electron](https://github.com/atom/electron) and [Polymer](https://www.polymer-project.org/1.0/).

- [x] Isolated app.  You need not prepare Chrome and can use favorite editor
- [x] Rich GitHub Flavored Markdown
  - code highlight
  - emoji
  - task list
  - links with tooltip
  - tree diagram and flowchart using [mermaid](https://github.com/knsv/mermaid)
  - math rendering using [katex](https://github.com/Khan/KaTeX)
- [x] Live reload
- [x] Automatic lint (remark-lint, markdownlint)
- [x] [Keyboard shortcuts](docs/shortcuts.md); All operations are ready for mouse and keyboard.
- [x] Both GUI and CLI friendly
- [x] Cross platform (OS X, Linux, Windows)
- [x] [Easy to install](docs/installation.md)
- [x] [Customizable with YAML config file](docs/customization.md)
  - keyboard shortcuts
  - linter
  - user CSS
  - code theme
  - etc...
- [x] [Search text in preview](docs/usage.md#search-text)
- [x] [Outline window](docs/usage.md#outline-window)
- [x] Print preview (to a paper / to a PDF file)
- [x] Support multi japanese encodings (utf8, utf16, sjis, euc-jp, unicode)
- [x] HTML preview
- [x] Dog-respected :dog2:


## Documents

All documents are in [docs](docs/) directory.  And I wrote [a Japanese blog post](http://rhysd.hatenablog.com/entry/2015/08/03/090646).

- [Installation](docs/installation.md)
- [Basic Usage](docs/usage.md)
- [Keyboard Shortcuts](docs/shortcuts.md)
- [Customization](docs/customization.md)
- [Tips](docs/tips.md)


## Overview

![main screenshot](https://raw.githubusercontent.com/rhysd/ss/master/Shiba/shiba-screenshot.gif)

1. At start up, Shiba is watching the current working directory (the path is in title of window).
2. When you edit the markdown file in current working directory, Shiba finds the update, renders the file in window and sets the result of lint.
3. You can see the result of lint by clicking the '!' button in menu.  When the button is red, it means that linter reported some errors.
4. You can change the watching directory/file using 'directory' button in menu or dropping file to window.
5. You can quit app by closing the window.

This is the simplest use case.  Please see [usage document](docs/usage.md) for more detail.


## Special Thanks

- The logo of this app came from [いらすとや](http://www.irasutoya.com/).
- This app was inspired by [@mattn](https://github.com/mattn)'s [mkup](https://github.com/mattn/mkup).
- This app referred [vmd](https://github.com/yoshuawuyts/vmd) a lot at first, which was a very simple markdown preview app built on Electron.
- Emoji pictures were from [github/gemoji](https://github.com/github/gemoji).
- The voice resource came from [効果音ラボ](http://soundeffect-lab.info/).


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


In packaged app, Electron part is licensed with below notice:

> Copyright (c) 2014 GitHub Inc.

Emojis were brought from gemoji project:

> octocat, squirrel, shipit
> Copyright (c) 2013 GitHub Inc. All rights reserved.
>
> bowtie, neckbeard, fu
> Copyright (c) 2013 37signals, LLC. All rights reserved.
>
> feelsgood, finnadie, goberserk, godmode, hurtrealbad, rage 1-4, suspect
> Copyright (c) 2013 id Software. All rights reserved.
>
> trollface
> Copyright (c) 2013 whynne@deviantart. All rights reserved.
>
> Source code:
>
> Copyright (c) 2013 GitHub, Inc.

