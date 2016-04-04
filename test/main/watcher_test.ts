/// <reference path="lib.d.ts" />

import {join} from 'path';

import * as assert from 'power-assert';
import * as touch from 'touch';
import WatchDog from '../../browser/watcher';
import DummyWebContents from './dummy_webcontents';
import {ipcMain as ipc} from 'electron';

const docdir = join(__dirname, '..', '..', 'doc');
const doc1 = join(docdir, 'ok.md');
const doc2 = join(docdir, 'notok.md');

const config = {
    linter: 'none',
    lint_options: {},
    file_ext: {
        markdown: ['md', 'markdown'],
    },
    ignore_path_pattern: '[\\\\/]\\.',
} as any;

context('WatchDog', () => {
    describe('#constructor', () => {
        it('generates watcher object with callbacks', () => {
            assert(new WatchDog(config));
        });
    });

    describe('#getDocumentKindFor', () => {
        const watchdog = new WatchDog(config);

        it('detects file extension from argument', () => {
            let kind: string;
            kind = watchdog.getDocumentKindFor('md');
            assert(kind === 'markdown');
            kind = watchdog.getDocumentKindFor('markdown');
            assert(kind === 'markdown');
        });

        it('returns undefined for unknown extension', () => {
            assert(watchdog.getDocumentKindFor('unknown-extension') === undefined);
        });
    });

    describe("Sending path to 'shiba:notify-path'", () => {
        it('ignores invalid path', () => {
            const watchdog = new WatchDog(config);
            watchdog.wakeup(new DummyWebContents() as any);
            ipc.emit('shiba:notify-path', {}, '');
            ipc.emit('shiba:notify-path', {}, 'invalid-path');
        });

        it('starts watcher on directory', () => {
            const c = new DummyWebContents() as any;
            c.once('shiba-notify-content-updated', () => {
                assert(false);
            });
            const watchdog = new WatchDog(config);
            watchdog.wakeup(c);
            ipc.emit('shiba:notify-path', {}, docdir);
            assert(watchdog.eyes);
            assert(watchdog.watching_path === docdir);
            watchdog.eyes.close();
        });

        it('starts watcher and notify first update on file path', done => {
            const watchdog = new WatchDog(config);
            const c = new DummyWebContents() as any;
            c.once('shiba:notify-content-updated', (_: any, kind: string, file: string) => {
                assert(kind === 'markdown');
                assert(file === doc1);
                assert(watchdog.watching_path === doc1);
                done();
            });
            watchdog.wakeup(c);
            ipc.emit('shiba:notify-path', {}, doc1);
            assert(watchdog.eyes);
        });
    });

    describe('path watcher', () => {
        const c = new DummyWebContents() as any;
        const watchdog = new WatchDog(config);
        watchdog.wakeup(c);

        it('watches file path', done => {
            ipc.emit('shiba:notify-path', {}, doc1);
            c.once('shiba:notify-content-updated', (_: any, kind: string, file: string) => {
                assert(kind === 'markdown');
                assert(file === doc1);
                watchdog.eyes.close();
                done();
            });

            setTimeout(() => touch(doc1), 100);
        });

        it('watches directory path', done => {
            ipc.emit('shiba:notify-path', {}, docdir);
            c.once('shiba:notify-content-updated', (_: any, kind: string, file: string) => {
                assert(kind === 'markdown');
                assert(file === doc1);
                watchdog.eyes.close();
                done();
            });

            setTimeout(() => touch(doc1), 100);
        });

        it('changes watching path', done => {
            ipc.emit('shiba:notify-path', {}, doc1);

            c.once('shiba:notify-content-updated', (_: any, kind: string, file: string) => {
                assert(kind === 'markdown');
                assert(file === doc2);
                watchdog.eyes.close();
                done();
            });
            ipc.emit('shiba:notify-path', {}, doc2);
        });
    });
});
