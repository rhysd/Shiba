Usage
=====

## Basic Usage

![Shiba on Linux](https://raw.githubusercontent.com/rhysd/screenshots/master/Shiba/shiba-main-0.1.0.png)

1. At Shiba starting up, it is watching the current working directory (watching directory is shown in title of window).
2. When you edit the markdown file in current working directory, shiba finds the update, renders the file in window and set the result of lint.
3. You can see the result of lint by pushing the '!' button in right above of window.  When the button is red, it means that linter reported some errors.
4. You can change the watching directory/file using 'directory' button in right above of window.  If you choose a file, Shiba watches the file only.  If you choose a directory, Shiba watches all files in the directory.  Wathing path is shown in title of window.
5. You can quit app by closing the window.

## Shortcuts

Keyboard shortcuts are available for above all operations.
Please refer [shortcuts document](shortcuts.md).

## Start up Options

You can specify initial path to watch as command line argument if you start Shiba from terminal.

```sh
# Linux
$ shiba {path}

# OS X
$ Shiba.app/Contents/MacOS/Shiba {path}

# Windows
$ shiba.exe {path}
```

The `{path}` is a path to markdown file or a directory you want to preview.

