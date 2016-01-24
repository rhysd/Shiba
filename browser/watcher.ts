import {app} from 'electron';
import * as marked from 'marked';
import * as path from 'path';
import * as fs from 'fs';
import * as chokidar from 'chokidar';
import {highlight} from 'highlight.js';
import {renderToString as katexRender} from 'katex';
import {replaceAll as replaceAllEmojis} from './emoji';
import * as config from './config'
import Linter from './linter';

marked.setOptions({
    highlight: function(code: string, lang: string): string {
        if (lang === undefined) {
            return code;
        }

        if (lang === 'mermaid') {
            return '<div class="mermaid">' + code + '</div>';
        }

        if (lang === 'katex') {
            return '<div class="katex">' + katexRender(code, {displayMode: true}) + '</div>';
        }

        try {
            return highlight(lang, code).value;
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

    constructor(
        public path: string,
        public render: (kind: string, content: Object) => void,
        public renderLintResult: (msgs: any[]) => void
    ) {
        this.config = config.load();
        this.linter = new Linter(this.config.linter, this.config.lint_options);

        console.log('Watcher starts with ' + this.path);

        this.startWatching();
    }

    private startWatching() {
        if (this.file_watcher) {
            this.file_watcher.close();
        }

        if (!fs.existsSync(this.path)) {
            return;
        }

        if (fs.statSync(this.path).isFile()) {
            this.sendUpdate(this.path);
        }

        const ext_pattern = Object.keys(this.config.file_ext)
                                .map((k: string) => this.config.file_ext[k].join('|'))
                                .join('|');

        const watched = path.join(this.path, '**', `*.(${ext_pattern})`);
        this.file_watcher = chokidar.watch(
            watched, {
                ignoreInitial: true,
                persistent: true,
                ignored: [new RegExp(this.config.ignore_path_pattern), /\.asar[\\\/]/]
            }
        );

        this.file_watcher.on('change', (file: string) => {
            console.log('File changed: ' + file);
            this.sendUpdate(file);
        });
        this.file_watcher.on('add', (file: string) => {
            console.log('File added: ' + file);
            this.sendUpdate(file);
        });
        this.file_watcher.on('error', (error: Error) => {
            console.log(`Error on watching: ${error}`);
        })
    }

    sendUpdate(file: string) {
        const ext = path.extname(file).substr(1);
        if (ext === '') {
            return;
        }

        const kind = (() => {
            for (const k in this.config.file_ext) {
                if (this.config.file_ext[k].indexOf(ext) !== -1) {
                    return k;
                }
            }
            return '';
        })();

        switch (kind) {
            case 'markdown': {
                // Encoding should be specified by config or detected
                fs.readFile(file, 'utf8', (err: NodeJS.ErrnoException, text: string) => {
                    if (err) {
                        console.log(`Can't open '${file}': ${err}`);
                        return;
                    }

                    app.addRecentDocument(file);

                    this.linter.lint(file, text, this.renderLintResult);

                    // Note:
                    // Replace emoji notations in HTML document because Markdown can't specify the size of image.
                    let html = marked(text);
                    this.render(kind, {
                        file: file,
                        html: replaceAllEmojis(html),
                    });
                });
                break;
            }
            case 'html': {
                // XXX: Temporary
                // I should send file name simply and renderer will read the file using <webview>
                this.render(kind, {file: file});
            }
        }
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
