![icon](assets/icon.iconset/icon_64x64.png) Shiba v2
====================================================
[![CI for v2][ci-badge]][ci]
[![dogs respected][shiba-badge]][shiba]

**:warning: WORK IN PROGRESS**

[Shiba][shiba] is a simple [Markdown][gh-markdown] preview application to be used with your favorite text editor.
It is designed for simplicity, performance, keyboard-friendliness.

![Screenshot of light/dark windows](https://raw.githubusercontent.com/rhysd/ss/master/Shiba/main.jpg)

Features:

- [GitHub-flavored Markdown][gfm] support; Emoji, Table, Math expressions with [Mathjax][mathjax], Diagrams with [mermaid.js][mermaid]
- Automatically update preview when the file is updated by efficiently watching files or directories using OS-specific filesystem
  events (FSEvents, inotify, ...)
- Automatically scroll to the last modified position
- All features can be accessed via keyboard shortcuts (scroll the article, search text, jump to section, ...). Type `?` to know
  all shortcuts
- Sections outline in side navigation bar. The current section is automatically focused
- Both CLI and GUI friendly; Available as a single binary executable as well as a desktop application installed to your system
- Performance critical part (parsing Markdown text, searching Markdown AST, calculating the last modified position, ...) and
  core application logic are written in [Rust][rust]. View logic is written in [TypeScript][ts] and [React][react]
- Cross platform; macOS, Windows, Linux are supported
- Customizable with [a YAML config file](./src/assets/default_config.yml) (color theme, keyboard shortcuts, custom CSS, ...)
- Dogs are respected :dog2:

## Installation

### Prerequisites

On Linux, some additional packages need to be installed via system package manager.

```sh
# On Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev libxdo-dev

# On Fedora
sudo dnf install gtk3-devel webkit2gtk4.1-devel libxdo
```

### Building from source

Install [Cargo package manager][cargo] via [rustup][] and [Node.js][nodejs] as prerequisites.

Clone this repository from GitHub:

```sh
git clone --depth=1 'https://github.com/rhysd/Shiba.git'
cd ./Shiba/v2
```

To build `shiba` (or `shiba.exe` on Windows) single binary executable as CLI application:

```sh
make release
cp ./target/release/shiba /path/to/your/bin/
shiba --help
```

To build macOS universal package:

```sh
make Shiba.dmg
```

To build an installer for Windows ([WiX v4][wix] is needed):

```sh
make shiba.msi
```

To build a package for Debian/Ubuntu ([cargo-deb][] is needed):

```sh
make shiba_0.0.0_amd64.deb
```

### Pre-built binaries

Download from [the release page](https://github.com/rhysd/Shiba/releases/tag/unreleased). Artifacts in this page are
**experimental** and built at some point of main branch. They may be buggy yet.

- `.zip` files are archived single binaries for each platforms
- `.msi`, `.deb`, `.dmg` files are installers for each platforms

**Note:** These executables are not signed. When your OS complains about it, go to 'Security' OS settings and allow the
executable to run.

## Documentation

Documentation is not ready yet.

## About v2

Shiba v2 is the complete rewrite of v1 with so many breaking changes. For v1, please visit [the `v1` branch][v1].

## License

This software is distributed under [the MIT license](./LICENSE).

[ci]: https://github.com/rhysd/Shiba/actions/workflows/ci.yml
[ci-badge]: https://github.com/rhysd/Shiba/actions/workflows/ci.yml/badge.svg
[shiba-badge]: https://img.shields.io/badge/dogs-respected-brightgreen.svg?longCache=true&style=flat
[shiba]: https://github.com/rhysd/Shiba
[gh-markdown]: https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax
[gfm]: https://github.github.com/gfm/
[mathjax]: https://www.mathjax.org/
[mermaid]: https://mermaid.js.org/
[rust]: https://www.rust-lang.org/ja
[ts]: https://www.typescriptlang.org/
[react]: https://react.dev/
[cargo]: https://doc.rust-lang.org/cargo/
[rustup]: https://rustup.rs/
[nodejs]: https://nodejs.org/en
[wix]: https://wixtoolset.org/
[cargo-deb]: https://github.com/kornelski/cargo-deb
[v1]: https://github.com/rhysd/Shiba/tree/v1
