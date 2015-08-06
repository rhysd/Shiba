// Note: Run from `npm test`

'use strict';

var electron = require('electron-prebuilt');
var spawnSync = require('child_process').spawnSync;
var join = require('path').join;

var args = process.argv.slice(2)
var idx_travis = args.indexOf('--travis');
var on_travis = idx_travis !== -1;

if (on_travis) {
    args.splice(idx_travis, 1);
}

function runOnElectron(tests) {
    var args = [join(__dirname, 'runner')].concat(tests);
    if (on_travis) {
        args.push('--travis');
    }
    var proc = spawnSync(electron, args, {stdio: 'inherit'});
    console.log(proc.status === 0 ? 'run.js: SUCCESS' : 'run.js: FAILED');
    process.exit(proc.status);
}

if (args.length === 0) {
    runOnElectron(join(__dirname, 'browser', 'out'));
} else {
    runOnElectron(join(process.cwd(), args));
}
