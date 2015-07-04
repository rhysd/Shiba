'use strict';

var ipc = require('ipc');
var marked = require('marked');
var path = require('path');
var fs = require('fs');

module.exports =
class Watcher {
    constructor(d, r) {
        this.dir = d;
        this.render = r;

        // XXX: Temporary
        const f = path.join(this.dir, 'README.md');
        if (fs.statSync(f).isFile()) {
            this.update(f);
        }
    }

    update(file) {
        let that = this;
        // Encoding should be specified by config or detected
        fs.readFile(file, 'utf-8', function(err, text){
            if (err) {
                console.log("Can't open: " + file);
                return;
            }

            that.render(marked(text));
        });
    }
};
