import * as fs from 'fs';
import * as path from 'path';
import {EventEmitter} from 'events';
import * as chokidar from 'chokidar';

const IGNORE_ASAR = /\.asar[\\\/]/;

export interface WatchingTarget {
    path: string;
    is_file: boolean;
}

export default class Watchdog extends EventEmitter {
    public target: WatchingTarget;
    private eyes: fs.FSWatcher;

    static create(id: number, p: string, c: AppConfig) {
        return new Promise<Watchdog>((resolve, reject) => {
            fs.stat(p, (err, stats) => {
                if (err) {
                    // e.g. Path does not exist
                    return reject(err);
                }
                resolve(new Watchdog(id, p, c, stats));
            });
        });
    }

    stop() {
        this.eyes.close();
        this.eyes = null;
    }

    shouldWatch(file: string) {
        for (const kind in this.config.file_ext) {
            for (const ext of this.config.file_ext[kind]) {
                if (file.endsWith('.' + ext)) {
                    return true;
                }
            }
        }
        return false;
    }

    emitUpdate(file: string) {
        if (this.target.is_file || this.shouldWatch(file)) {
            this.emit('update', file);
        }
    }

    getWatchingPattern() {
        if (this.target.is_file) {
            if (this.shouldWatch(this.target.path)) {
                return this.target.path;
            } else {
                return null;
            }
        } else {
            const ext_pattern = Object.keys(this.config.file_ext).map((k: string) => this.config.file_ext[k].join('|')).join('|');
            return path.join(this.target.path, '**', `*.(${ext_pattern})`);
        }
    }

    start() {
        return new Promise((resolve, reject) => {
            const pattern = this.getWatchingPattern();
            if (pattern === null) {
                return reject(new Error(`'${this.target.path}' is not a target to watch`));
            }

            if (this.started()) {
                this.stop();
            }

            const eyes = chokidar.watch(pattern, {
                    ignoreInitial: true,
                    persistent: true,
                    ignored: [new RegExp(this.config.ignore_path_pattern), IGNORE_ASAR],
                });

            eyes.on('change', this.emitUpdate);
            eyes.on('add', this.emitUpdate);
            eyes.on('error', (e: Error) => this.emit('error', e));
            eyes.on('ready', () => {
                this.emit('ready');
                resolve(this);
            });

            this.eyes = eyes;
        });
    }

    started() {
        return this.eyes !== null;
    }

    private constructor(public id: number, watching: string, public config: AppConfig, stats: fs.Stats) {
        super();
        this.target = {
            path: watching,
            is_file: stats.isFile(),
        };
        this.eyes = null;
        this.emitUpdate = this.emitUpdate.bind(this);
    }
}


