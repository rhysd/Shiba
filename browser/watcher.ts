/// <reference path="lib.d.ts" />

import {app, ipcMain as ipc} from 'electron';
import * as path from 'path';
import * as fs from 'fs';
import * as assertTruthy from 'assert';
import * as chokidar from 'chokidar';
import Linter from './linter';

export default class WatchDog {
    public sender: Electron.WebContents;
    public watching_path: string;
    public eyes: fs.FSWatcher;
    public linter: Linter;

    constructor(public config: Config) {
    }

    wakeup(sender: Electron.WebContents) {
        this.sender = sender;
        ipc.on('shiba:notify-path', (_: any, new_path: string) => {
            this.setWatchingPath(new_path);
        });
        ipc.on('shiba:request-path', () => {
            this.sender.send('shiba:return-path', this.watching_path);
        });
        this.linter = new Linter(this.sender, this.config.linter, this.config.lint_options);
    }

    getDocumentKindFor(ext: string) {
        for (const k in this.config.file_ext) {
            if (this.config.file_ext[k].indexOf(ext) !== -1) {
                return k;
            }
        }
        return undefined;
    }

    statWatchingPath() {
        try {
            return fs.lstatSync(this.watching_path);
        } catch (e) {
            // Path is not found
            return null;
        }
    }

    openEyes(pattern: string) {
        const followSymlinks = !!(this.config.path_watcher || {} as PathWatcherConfig).follow_symlinks;
        const eyes = chokidar.watch(pattern, {
                ignoreInitial: true,
                persistent: true,
                ignored: [new RegExp(this.config.ignore_path_pattern), /\.asar[\\\/]/],
                followSymlinks,
            });

        eyes.on('change', (file: string) => {
            console.log('File changed: ' + file);
            this.sendContentUpdated(file);
        });
        eyes.on('add', (file: string) => {
            console.log('File added: ' + file);
            this.sendContentUpdated(file);
        });
        eyes.on('error', (error: Error) => {
            console.log(`Error on watching: ${error.message}`);
        });

        return eyes;
    }

    setWatchingPath(new_path: string) {
        if (new_path === this.watching_path || new_path === '') {
            return;
        }

        if (this.watching_path === undefined) {
            console.log(`Start watching path: ${new_path}`);
        } else {
            console.log(`Change watching path: ${this.watching_path} -> ${new_path}`);
        }

        this.watching_path = new_path;

        if (this.eyes) {
            this.eyes.close();
        }

        const stats = this.statWatchingPath();
        if (stats === null) {
            return;
        }

        const path_is_file = stats.isFile();
        if (path_is_file) {
            this.sendContentUpdated(this.watching_path);
        }

        assertTruthy(path_is_file || stats.isDirectory());

        const ext_pattern = Object.
                keys(this.config.file_ext).
                map((k: string) => this.config.file_ext[k].join('|')).
                join('|');

        const watched = path_is_file ?
                this.watching_path :
                path.join(this.watching_path, '**', `*.(${ext_pattern})`);

        this.eyes = this.openEyes(watched);
    }

    sendContentUpdated(file: string) {
        const ext = path.extname(file).substr(1);
        if (ext === '') {
            return;
        }

        const kind = this.getDocumentKindFor(ext);
        switch (kind) {
            case 'markdown': {
                app.addRecentDocument(file);
                this.sender.send('shiba:notify-content-updated', kind, file);
                this.linter.lint(file);
                break;
            }
            case 'html': {
                app.addRecentDocument(file);
                this.sender.send('shiba:notify-content-updated', kind, file);
                break;
            }
            default: {
                // Do nothing
                break;
            }
        }
    }

    hasStarted() {
        return this.sender !== undefined;
    }
}

