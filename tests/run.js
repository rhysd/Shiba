// Note: Run from `npm test`

'use strict';

var electron = require('electron-prebuilt');
var spawnSync = require('child_process').spawnSync;

var join = require('path').join;
var fs = require('fs');
var glob = require('globby');

function runOnElectron(tests) {
    var args = [join(__dirname, 'runner')].concat(tests);
    var proc = spawnSync(electron, args, {stdio: 'inherit'});
    console.log(proc.status === 0 ? 'run.js: SUCCESS' : 'run.js: FAILED');
}

var args = process.argv.slice(2)
if (args.length === 0) {
    runOnElectron([join(__dirname, 'browser'), join(__dirname, 'renderer')]);
} else {
    runOnElectron(join(process.cwd(), args));
}
