Install Shiba
=============

Installing and preparing Shiba is very easy.
You can install Shiba in 3 ways.
Almost all of you should follow [general way](#general).  If you want to manage Shiba with [npm](https://www.npmjs.com/), please consider to use [npm way](#npm)


## <a name="general"> General

Shiba is released for Linux, OS X and Windows.
Please download zip file from [release page](https://github.com/rhysd/Shiba/releases) and unzip the archive.
Note that the file is a bit large because it includes a Chromium to execute Electron app.

### Linux

You can find `shiba` executable in the directory.  Please simply execute it.

### OS X

You can find `Shiba.app` in the directory.
Please simply execute `open -a Shiba` to execute Shiba from command line.  Note that `open` command can't maintain the command line arguments.  If you want to pass command line arguments, please use executable `Shiba.app/Contents/MacOS/Shiba` directly.

If you want to put Shiba.app in Dock and start with double click, put `Shiba.app` to your Application directory (`~/Applications` or `/Applications`).

And you can also use [Homebrew Cask](https://caskroom.github.io/) to install Shiba.

```
$ brew cask install shiba
```

### Windows

You can find `shiba.exe` in the directory.  Please simply use it by double click or from command prompt.  You need to install nothing.


## <a name="npm"> Via npm

I already registered Shiba to [npm](https://www.npmjs.com/).  You can install it via npm as below.

```
$ npm install -g shiba
```

Then you can simply execute `shiba` command from command line.


## For development

```sh
$ git clone https://github.com/rhysd/Shiba.git && cd Shiba

$ gem install slim
$ npm install -g typescript bower
$ rake dep

# Build Shiba
$ rake build

# Execute Shiba
$ ./bin/cli.js README.md

# With DevTools
$ NODE_ENV=development ./bin/cli.js README.md

# Watch and differential build
$ rake watch
```


-----------------
[installation](installation.md) | [usage](usage.md) | [customization](customization.md) | [shortcuts](shortcuts.md) | [tips](tips.md)
