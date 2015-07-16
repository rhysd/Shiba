Install Shiba
=============

Installing and preparing Shiba is very easy.
You can install Shiba in 3 ways.
Almost all of you should follow [General way](#general).  If you want to avoid downloading large chromium package, please consider to use [npm way](#npm)

## <a name="general"> General

Experimental alpha release is available for Linux, OS X and Windows.
Please download from [release page](https://github.com/rhysd/Shiba/releases) and unzip the archive.
Note that the file is a bit large because it includes chromium to execute Electron app.

### Linux

You can find executable `shiba` in the directory.  Please simply execute it.

### OS X

You can find `Shiba.app` in the directory.
Please simply execute `open -a Shiba` to execute Shiba from command line.  Note that `open` command can't maintain the command line arguments.  If you want to pass command line arguments, please use executable `Shiba.app/Contents/MacOS/Shiba` directly.

If you want to put Shiba.app in Dock and start with double click, put `Shiba.app` to your Application directory (`~/Applications` or `/Applications`).

### Windows

You can find `shiba.exe` in the directory.  Please simply use it by double click or from command prompt.  You need to install nothing.



## <a name="npm"> Via npm

I already registered Shiba to [npm](https://www.npmjs.com/).  You can install it via npm as below.  Note that you must install `electron-prebuilt` in advance.

```
$ npm install -g electron-prebuilt
$ npm install -g shiba
```

Then you can simply execute `shiba` command from command line.



## For development

To execute Shiba in git repository for development purpose, please use `electron` command which is available after installing `electron-prebuilt` via npm.

```sh
$ git clone https://github.com/rhysd/Shiba.git && cd Shiba
$ bower install && npm install
$ electron . # Or `electron . {file to watch}`
```


