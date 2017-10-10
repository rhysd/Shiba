#! /usr/bin/env node

const child_process = require('child_process');
const electron = require('electron');
const join = require('path').join;

const argv = [join(__dirname, '..')];

const help = process.argv.indexOf('--help') !== -1 || process.argv.indexOf('-h') !== -1;
if (help) {
    console.log(`Shiba: Rich markdown previewer

Usage:

    $ shiba [options] [path]

    Shiba watches the path to detect file changes and automatically updates
    the preview.
    If the path represents a directory, Shiba will watch all markdown files in
    it recursively. If the path is a file, Shiba will watch only it.
    When no path is given, Shiba will watches 

    Please read documents bundled in package or visit following links to know
    more details (i.e. configurations).

    - https://github.com/rhysd/Shiba
    - https://github.com/rhysd/Shiba/blob/master/docs


Options:

    --detach        Detach the application process. It will discard stdout and
                    stderr outputs.
    --version       Show the application and frameworks versions.
    -h | --help     Show this help.
`);
    process.exit(0);
}

const detach_idx = process.argv.indexOf('--detach');
const detached = detach_idx !== -1;
if (detached) {
    process.argv.splice(detach_idx, 1);
}

const version_idx = process.argv.indexOf('--version');
const version = version_idx !== -1;
if (version) {
    process.argv.splice(detach_idx, 1);
    argv.push('--version');
}

const len = process.argv.length;

// First is 'node' and Second arg is '/path/to/bin/shiba'.
// If user specifies argument, the length of argv must be more than 2.
if (len > 2) {
    argv.push(process.argv[len-1]);
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
