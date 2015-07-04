'use strict';

var ipc = require('ipc');
var marked = require('marked');
var path = require('path');
var fs = require('fs');

// Note:
// ES6 class syntax is unavailable for 'remote' module in renderer process
function Watcher(d, r) {
    this.dir = d;
    this.render = r;

    // XXX: Temporary
    const f = path.join(this.dir, 'README.md');
    if (fs.statSync(f).isFile()) {
        this.update(f);
    }
}

Watcher.prototype.update = function(file) {
    let that = this;
    // Encoding should be specified by config or detected
    fs.readFile(file, 'utf-8', function(err, text){
        if (err) {
            console.log("Can't open: " + file);
            return;
        }

        that.render(marked(text));
    });
};

Watcher.prototype.changeWatchingDir = function(new_dir) {
    if (new_dir === this.dir) {
        return;
    }

    this.dir = new_dir;
    // TODO
};

module.exports = Watcher;
