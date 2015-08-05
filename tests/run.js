// Note: Run from `npm test`

'use strict';

var electron = require('electron-prebuilt');
var spawnSync = require('child_process').spawnSync;
var join = require('path').join;

function runOnElectron(tests) {
    var args = [join(__dirname, 'runner')].concat(tests);
    var proc = spawnSync(electron, args, {stdio: 'inherit'});
    console.log(proc.status === 0 ? 'run.js: SUCCESS' : 'run.js: FAILED');
    process.exit(proc.status);
}

var args = process.argv.slice(2)
if (args.length === 0) {
    runOnElectron(join(__dirname, 'browser', 'out'));
} else {
    runOnElectron(join(process.cwd(), args));
}
