Installation
============

This document describes how to install Shiba.

## Runtime dependencies

Shiba uses platform-specific WebView runtimes through [wry][] crate. You need to install runtime dependencies on some
platforms.

### Ubuntu or Debian

On Linux, Shiba uses GTKWebKit. Some additional shared libraries need to be installed via system package manager.

```sh
# On Ubuntu or Debian
sudo apt install libwebkit2gtk-4.1-dev libxdo-dev libgtk-3-dev

# On Fedora
sudo dnf install gtk3-devel webkit2gtk4.1-devel libxdo

# On Arch Linux
sudo pacman -S webkit2gtk-4.1 gtk3 xdotool
```

Note: If you install `.deb` file through `dpkg` command, these dependencies are automatically installed.

### Windows

Shiba uses [WebView2 component][webview2]. Usually it is already installed because Windows 11 and recent Windows 10
installs it by default. When you see Shiba crashes on startup, please check if WebView2 is installed.

### macOS

Shiba uses WKWebView and it is installed by default on macOS. No additional dependency is necessary.

## Use Pre-built binaries

**:warning:These released artifacts are still experimental and built at some point of `main` branch. They may be buggy.**

Pre-built binaries are hosted on [the release page](https://github.com/rhysd/Shiba/releases/tag/unreleased).

- `.zip` files are archived single binaries for each platforms. Put the executable binary to your `$PATH` directory
- `.msi` file is a Windows installer. Double-click it and follow the instructions
- `.deb` file is a package for Debian and Ubuntu. Install it via `dpkg` and manage it with `apt`
- `.dmg` file is a universal macOS package which supports both M1 Mac and Intel Mac. Double-click it and move the `.app`
  file to `Applications` directory

**Note:** These executables are not signed. When your OS complains about it, go to 'Security' OS settings and allow the
executable to run.

## Build from source

### Preparation

Shiba is built with Rust and TypeScript. Install [Cargo package manager][cargo] via [rustup][] and [Node.js][nodejs] as
build toolchain. Ensure that the latest stable Rust toolchain is installed.

All build tasks are defined as `make` rules. On Windows, install Make via [winget][winget-make] or [chocolatey][choco-make].

Finally clone the Shiba Git repository from GitHub.

```sh
git clone --depth=1 'https://github.com/rhysd/Shiba.git'
cd ./Shiba/v2
```

### Build single-binary executable

To build `shiba` (or `shiba.exe` on Windows) single-binary executable as CLI application, run `make release`.
It generates an executable in `target/release` directory. Put it to your `$PATH` directory.

```sh
make release
./target/release/shiba --help
```

### Build macOS universal application

Targets for both M1 Mac and Intel Mac are necessary. Install them via rustup:

```sh
rustup target add x86_64-apple-darwin aarch64-apple-darwin
```

`make` does everything to generate `Shiba.dmg` or `Shiba.app`.

```sh
make Shiba.dmg
# or
make Shiba.app
```

### Build Windows installer

[WiX v4][wix] is needed. Ensure `wix` command works in your terminal.

`make` does everything to generate `shiba.msi` installer file.

```sh
make shiba.msi
```

### Build Debian or Ubuntu package

[cargo-deb][] is needed. Ensure `cargo deb` command works in your terminal.

`make` does everything to generate `shiba_amd64.deb` package file.

```sh
make shiba_amd64.deb
```

[wry]: https://github.com/tauri-apps/wry
[webview2]: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
[cargo]: https://doc.rust-lang.org/cargo/
[rustup]: https://rustup.rs/
[nodejs]: https://nodejs.org/en
[winget-make]: https://winget.run/pkg/GnuWin32/Make
[choco-make]: https://community.chocolatey.org/packages/make
[wix]: https://wixtoolset.org/
[cargo-deb]: https://github.com/kornelski/cargo-deb
