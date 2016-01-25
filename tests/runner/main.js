'use strict';

var electron = require('electron');
var app = electron.app;
var BrowserWindow = electron.BrowserWindow;
var ipc = electron.ipcMain;

var glob = require('globby').sync;
var path = require('path');
var fs = require('fs');
var Mocha = require('mocha');
var chai = require('chai');

const RE_JS = /\.js$/;
const RE_RENDERER = /renderer$/;

var on_travis = false;
if (process.argv[process.argv.length-1] == '--travis') {
    on_travis = true;
    process.argv.pop();
}

var renderer_test_paths = [];
var renderer_test_exit_status = null;

function addBrowserTest(p, mocha) {
    const stats = fs.statSync(p);
    if (stats.isDirectory()) {
        for (const f of glob(path.join(p, '*'))) {
            addBrowserTest(f, mocha);
        }
    } else if (stats.isFile() && RE_JS.test(p)) {
        mocha.addFile(p);
    }
}

function addRendererTest(p) {
    if (!on_travis) {
        renderer_test_paths = renderer_test_paths.concat(glob(path.join(p, '**', '*.js')))
    }
}

function fetchRendererTests() {
    if (renderer_test_paths.length === 0) {
        return;
    }

    let w = new BrowserWindow({width: 800, height: 600});
    const html = 'file://' + path.resolve(__dirname, 'index.html');
    w.on('closed', function() { process.exit(0) });
    w.webContents.on('dom-ready', function(){ w.send('renderer-test', {files: renderer_test_paths}); w.openDevTools();});
    w.loadURL(html);

    ipc.on('renderer-test-result', function(event, exit_status){
        renderer_test_exit_status = exit_status;
    });
}

app.on('ready', function() {
    let mocha = new Mocha();
    global.assert = chai.assert;
    global.on_travis = on_travis;

    for (const path of process.argv.slice(2)) {
        if (RE_RENDERER.test(path)) {
            addRendererTest(path);
        } else {
            addBrowserTest(path, mocha);
        }
    }

    fetchRendererTests();

    mocha.ui('bdd').run(function(failures) {
        process.on('exit', function() {
            if (typeof renderer_test_exit_status === 'number') {
                process.exit(failures + renderer_test_exit_status);
            } else {
                process.exit(failures);
            }
        });
        if (renderer_test_paths.length === 0) {
            app.quit();
        }
    });
});
