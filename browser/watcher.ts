import {app} from 'electron';
import * as path from 'path';
import * as fs from 'fs';
import * as chokidar from 'chokidar';
import * as config from './config';
import Linter from './linter';

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

        console.log('Start to watch ' + this.path);

        this.startWatching();
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
                app.addRecentDocument(file);
                this.render(kind, file);
                this.linter.lint(file, this.renderLintResult);
                break;
            }
            case 'html': {
                app.addRecentDocument(file);
                this.render(kind, file);
                break;
            }
            default: {
                // Do nothing
                break;
            }
        }
    }

    changeWatchingDir(new_path: string) {
        if (new_path === this.path) {
            return;
        }

        console.log(`Change watching path ${this.path} -> ${new_path}`);

        this.path = new_path;
        this.startWatching();
    }

    getLintRuleURL() {
        return this.linter.lint_url;
    }

    private startWatching() {
        if (this.file_watcher) {
            this.file_watcher.close();
        }

        if (!fs.existsSync(this.path)) {
            return;
        }

        const is_file = fs.statSync(this.path).isFile();
        if (is_file) {
            this.sendUpdate(this.path);
        }

        const ext_pattern = Object.keys(this.config.file_ext)
                                .map((k: string) => this.config.file_ext[k].join('|'))
                                .join('|');

        const watched = is_file ? this.path : path.join(this.path, '**', `*.(${ext_pattern})`);
        this.file_watcher = chokidar.watch(
            watched, {
                ignoreInitial: true,
                persistent: true,
                ignored: [new RegExp(this.config.ignore_path_pattern), /\.asar[\\\/]/],
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
        });
    }
}

export = Watcher;
