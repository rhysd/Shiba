'use strict';

let ipc = require('ipc');
let marked = require('marked');
let path = require('path');
let fs = require('fs');
let chokidar = require('chokidar');
let markdownlint = require('markdownlint');

// Note:
// ES6 class syntax is unavailable for 'remote' module in renderer process
function Watcher(p, r, l) {
    this.path = p;
    this.render = r;
    this.renderLintResult = l;

    console.log('Watcher starts with ' + p);

    this.startWatching();
}

Watcher.prototype.startWatching = function() {
    if (this.file_watcher) {
        this.file_watcher.close();
    }

    if (!fs.existsSync(this.path)) {
        return;
    }

    if (fs.statSync(this.path).isFile()) {
        this._sendUpdate(this.path);
    }

    this.file_watcher = chokidar.watch(this.path);
    let that = this;
    this.file_watcher.on('change', function(file){
        console.log('File changed: ' + file);
        that._sendUpdate(file);
    });
};

Watcher.prototype._sendUpdate = function(file) {
    if (!/.+\.(md|mkd|markdown)$/.test(file)) {
        return;
    }

    let that = this;

    // Encoding should be specified by config or detected
    fs.readFile(file, 'utf8', function(err, text){
        if (err) {
            console.log("Can't open: " + file);
            return;
        }

        let options = {
            // TODO: Enable to specify lint configurations
            'strings' : {}
        };
        options.strings[path.basename(file)] = text;

        markdownlint(options, function(err, result){
            if (!err) {
                that.renderLintResult(result.toString());
            }
        });

        that.render(marked(text));
    });
};

Watcher.prototype.changeWatchingDir = function(new_path) {
    if (new_path === this.path) {
        return;
    }

    console.log('Change watching path' + this.path + ' -> ' + new_path);

    this.path = new_path;
    this.startWatching();
};

module.exports = Watcher;
