Usage
=====

## Basic Usage

Click application icon to start Shiba.  Below is an OS X example.

![dock startup](https://raw.githubusercontent.com/rhysd/ss/master/Shiba/dock.png)

Then you can see an empty window.  At first, find the 'directory' icon and push it.  A dialog will be shown and you can specify a directory or markdown file to make Shiba watch.

![directory icon](https://raw.githubusercontent.com/rhysd/ss/master/Shiba/menu-no-error.png)

When specifying a markdown file, Shiba will show the preview of it.  Below window is a preview of [this file](https://gist.github.com/rhysd/ffe61ad01f9a7a9fe69f).

![main window](https://raw.githubusercontent.com/rhysd/ss/master/Shiba/window-main.png)

After that, when you edit some lines of the file, the preview will be automatically updated.  So you can write your markdown document with checking preview.

If you want to change the watching directory/file, please push the 'directory' button again.  And you can quit app by closing the window.


## Lint

Shiba has integrated markdown linter.  When file is updated, Shiba will run linter automatically and report it if an error occurs.  You can access the lint result by '!' button in right above of the window.
At first, the '!' button is normal color as below.

![no error](https://raw.githubusercontent.com/rhysd/ss/master/Shiba/menu-no-error.png)

When linter reports some errors, the button's color would be changed to red.

![error](https://raw.githubusercontent.com/rhysd/ss/master/Shiba/menu-errors.png)

When you want to know the detail of lint errors, simply click the red button.  It shows the list of errors.

![lint result](https://raw.githubusercontent.com/rhysd/ss/master/Shiba/window-lint.png)

Shortcut `CTRL + L` is also available to toggle lint result drawer.

## Shortcuts

Keyboard shortcuts are available for above all operations.
Please refer [shortcuts document](shortcuts.md).

## Links in Documents

### Clicking a link to local markdown file

When you click a link to local markdown document, Shiba shows the preview of the document _temporarily_.
'_temporarily_' means that Shiba still watches the original document before jump the link.  So you can back to the original document when original document is updated or 'reload' button is pushed.

### Clicking an external link

When you click an external link (which starts with `http://` or `https://`), Shiba tends to open the link with external browser.  This behavior is mandatory in terms of security because pages in Electron are loaded outside sandbox.

### Clicking a `#hash` link

When you click to the internal link to hash, page simply scrolls to the target.

### Clicking a link to local markdown file with modifier key

If you click a link to local markdown document with modifier key (Ctrl or Command), Shiba changes the watching directory to the linked document.

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


-----------------
[installation](installation.md) | [usage](usage.md) | [customization](customization.md) | [shortcuts](shortcuts.md) | [tips](tips.md)
