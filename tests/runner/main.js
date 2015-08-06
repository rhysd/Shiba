'use strict';

var app = require('app');
var BrowserWindow = require('browser-window');
var glob = require('globby').sync;
var join = require('path').join;
var fs = require('fs');
var Mocha = require('mocha');
var chai = require('chai');

const RE_JS = /\.js$/;

var on_travis = false;
if (process.argv[process.argv.length-1] == '--travis') {
    on_travis = true;
    process.argv.pop();
}

function addTest(path, mocha) {
    const stats = fs.statSync(path);
    if (stats.isDirectory()) {
        var index_file = join(path, 'index.html');
        if (fs.existsSync(index_file)) {
            // TODO
            return;
        } else {
            for (const f of glob(join(path, '*'))) {
                addTest(f, mocha);
            }
            return;
        }
    } else if (stats.isFile() && RE_JS.test(path)) {
        mocha.addFile(path);
    }
}

app.on('ready', function() {
    let mocha = new Mocha();
    global.assert = chai.assert;
    global.on_travis = on_travis;
    for (const path of process.argv.slice(2)) {
        addTest(path, mocha);
    }
    mocha.ui('bdd').run(function(failures) {
        process.on('exit', function() {
            process.exit(failures);
        });
        app.quit();
    });
});
