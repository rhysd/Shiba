#! /usr/bin/env node

'use strict';

var child_process = require('child_process');
var electron = require('electron-prebuilt');
var join = require('path').join;
var existsSync = require('fs').existsSync;

var argv = [join(__dirname, '..')];

var detach_idx = process.argv.indexOf('--detach');
var detached = detach_idx !== -1;
if (detached) {
    process.argv.splice(detach_idx, 1);
}

var version_idx = process.argv.indexOf('--version');
var version = version_idx !== -1;
if (version) {
    process.argv.splice(detach_idx, 1);
    argv.push('--version');
}

var len = process.argv.length;

// First is 'node' and Second arg is '/path/to/bin/shiba'.
// If user specifies argument, the length of argv must be more than 2.
if (len > 2) {
    var last_arg = process.argv[len-1];
    if (existsSync(last_arg)) {
        argv.push(last_arg);
    } else {
        argv.push(process.cwd());
    }
} else {
    argv.push(process.cwd());
}

if (detached && !version) {
    child_process.spawn(electron, argv, {
        stdio: 'ignore',
        detached: true
    }).unref();
} else {
    child_process.spawn(electron, argv, {
        stdio: 'inherit'
    });
}
