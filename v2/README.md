![icon](assets/icon.iconset/icon_64x64.png) Shiba v2
====================================================
[![CI for v2][ci-badge]][ci]
[![dogs respected][shiba-badge]][shiba]

**:warning: WORK IN PROGRESS**

[Shiba][shiba] is a simple [Markdown][gh-markdown] preview application to be used with your favorite text editor.
It is designed for simplicity, performance, keyboard-friendliness.

![Screenshot of light/dark windows](https://raw.githubusercontent.com/rhysd/ss/master/Shiba/main.jpg)

Features:

- [GitHub-flavored Markdown][gfm] support; Emoji, Table, Math expressions with [Mathjax][mathjax], Diagrams with [mermaid.js][mermaid], ...
- Automatically update preview when the file is updated by efficiently watching files or directories using OS-specific filesystem
  events (FSEvents, inotify, ...)
- Automatically scroll to the last modified position
- All features can be accessed via keyboard shortcuts (scroll the article, search text, jump to section, ...). Type `?` to know
  all shortcuts
- Sections outline in side navigation bar. The current section is automatically focused
- Both CLI and GUI friendly; Available as a single binary executable as well as a desktop application installed to your system
- Performance critical part (parsing Markdown text, searching Markdown AST, calculating the last modified position, ...) and
  core application logic are written in [Rust][rust]. View logic written in [TypeScript][ts] and [React][react] runs on
  platform-specific WebView
- Cross platform; macOS, Windows, Linux are supported
- Customizable with [a YAML config file](./src/assets/default_config.yml) (color theme, keyboard shortcuts, custom CSS, ...)
- Dogs are respected :dog2:

## Documentation

All documentations are in the [docs](./docs) directory.

- [Installation](./docs/installation.md)
- ...More docs will be added

## About v2

Shiba v2 is the complete rewrite of v1 using Rust and platform-specific WebView. For v1, please visit [the `v1` branch][v1].

## License

This software is distributed under [the MIT license](./LICENSE).

[ci]: https://github.com/rhysd/Shiba/actions/workflows/watchdogs.yml
[ci-badge]: https://github.com/rhysd/Shiba/actions/workflows/watchdogs.yml/badge.svg
[shiba-badge]: https://img.shields.io/badge/dogs-respected-brightgreen.svg?longCache=true&style=flat
[shiba]: https://github.com/rhysd/Shiba
[gh-markdown]: https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax
[gfm]: https://github.github.com/gfm/
[mathjax]: https://www.mathjax.org/
[mermaid]: https://mermaid.js.org/
[rust]: https://www.rust-lang.org/ja
[ts]: https://www.typescriptlang.org/
[react]: https://react.dev/
[v1]: https://github.com/rhysd/Shiba/tree/v1
