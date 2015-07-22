import app = require('app');
import ipc = require('ipc');
import marked = require('marked');
import path = require('path');
import fs = require('fs');
import chokidar = require('chokidar');
import hljs = require('highlight.js');
import emoji = require('./emoji');
import config = require('./config');
import Linter = require('./linter');

marked.setOptions({
    highlight: function(code: string, lang: string): string {
        if (lang === undefined) {
            return code;
        }

        try {
            return hljs.highlight(lang, code).value;
        } catch (e) {
            console.log(e.message);
            return code;
        }
    }
});

class Watcher {
    config: config.Config;
    linter: Linter;
    file_watcher: fs.FSWatcher;

    constructor(public path, public render, public renderLintResult) {
        this.config = config.load();
        this.linter = new Linter(this.config.linter, this.config.lint_options);

        console.log('Watcher starts with ' + this.path);

        this.startWatching();
    }

    startWatching() {
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
                ignoreInitial: true,
                persistent: true,
                ignored: /[\\\/]\./
            }
        );

        this.file_watcher.on('change', (file: string) => {
            console.log('File changed: ' + file);
            this._sendUpdate(file);
        });
        this.file_watcher.on('add', (file: string) => {
            console.log('File added: ' + file);
            this._sendUpdate(file);
        });
    }

    _sendUpdate(file: string) {
        if (!/.+\.(md|mkd|markdown)$/.test(file)) {
            return;
        }

        // Encoding should be specified by config or detected
        fs.readFile(file, 'utf8', (err: NodeJS.ErrnoException, text: string) => {
            if (err) {
                console.log("Can't open: " + file);
                return;
            }

            app.addRecentDocument(file);

            this.linter.lint(path.basename(file), text, this.renderLintResult);

            // Note:
            // Replace emoji notations in HTML document because Markdown can't specify the size of image.
            let html = marked(text);
            this.render(emoji.replaceAll(html));
        });
    }

    changeWatchingDir(new_path: string) {
        if (new_path === this.path) {
            return;
        }

        console.log('Change watching path' + this.path + ' -> ' + new_path);

        this.path = new_path;
        this.startWatching();
    }

    getLintRuleURL() {
        return this.linter.lint_url;
    }
}

export = Watcher;
