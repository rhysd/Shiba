#! /usr/bin/env node

'use strict';

var child_process = require('child_process');
var electron = require('electron-prebuilt');
var path = require('path');
var fs = require('fs');

var argv = [path.join(__dirname, '..')];
var len = process.argv.length;

// First is 'node' and Second arg is '/path/to/bin/shiba'.
// If user specifies argument, the length of argv must be more than 2.
if (len > 2) {
    var last_arg = process.argv[len-1];
    if (fs.existsSync(last_arg)) {
        argv.push(last_arg);
    } else {
        argv.push(process.cwd());
    }
} else {
    argv.push(process.cwd());
}

child_process.spawn(electron, argv, {stdio: 'inherit'});
