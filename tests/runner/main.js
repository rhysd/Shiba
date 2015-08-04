'use strict';

var app = require('app');
var BrowserWindow = require('browser-window');
var glob = require('globby').sync;
var join = require('path').join;
var fs = require('fs');

var mainWindow = null;

function runRendererTest(html_file) {
    console.log('Start renderer process test: ' + html_path);
    // TODO
    return true;
}

function runBrowserTest(js_file) {
    console.log('Start browser process test: ' + js_file);
    return require(js_file)();
}

function runTest(path) {
    var stats = fs.statSync(path);
    if (stats.isDirectory()) {
        var index_file = join(path, 'index.html');
        if (fs.existsSync(index_file)) {
            return runRendererTest(index_file);
        } else {
            let success = true;
            for (const f of glob(join(path, '*'))) {
                success = runTest(f);
            }
            return success;
        }
    } else if (stats.isFile()) {
        return runBrowserTest(path)
    } else {
        console.log('Ignored: Target does not exist: ' + path);
        return true;
    }
}

app.on('ready', function() {
    let success = true;
    for (const path of process.argv.slice(2)) {
        success = runTest(path);
    }

    app.quit();
    mainWindow = new BrowserWindow({});
});
