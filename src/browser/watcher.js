'use strict';

let ipc = require('ipc');
let marked = require('marked');
let path = require('path');
let fs = require('fs');
let chokidar = require('chokidar');
let markdownlint = require('markdownlint');
let hljs = require('highlight.js');
let emoji = require('./emoji.js');
let config = require('./config.js');
let Linter = require('./linter.js');

marked.setOptions({
    highlight: function(code, lang) {
        if (lang === undefined) {
            return code;
        }

        return hljs.highlight(lang, code).value;
    }
});

// Note:
// ES6 class syntax is unavailable for 'remote' module in renderer process
function Watcher(p, r, l) {
    this.path = p;
    this.render = r;
    this.renderLintResult = l;
    this.config = config.load();
    this.linter = new Linter(this.config.linter);

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

    const ext_pattern = `*.(${this.config.file_ext.join('|')})`;
    const watched = path.join(this.path, '**', ext_pattern);
    this.file_watcher = chokidar.watch(
        watched, {
            ignored: /[\\\/]\./,
            persistent: true
        }
    );
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

        that.linter.lint(path.basename(file), text, that.renderLintResult);

        // Note:
        // Replace emoji notations in HTML document because Markdown can't specify the size of image.
        let html = marked(text);
        that.render(emoji.replaceAll(html));
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

Watcher.prototype.getLintRuleURL = function() {
    return this.linter.lint_url;
};

module.exports = Watcher;
