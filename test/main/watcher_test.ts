/// <reference path="lib.d.ts" />

import {join} from 'path';

import * as assert from 'power-assert';
import * as touch from 'touch';
import Watcher = require('../../browser/watcher');

const docdir = join(__dirname, '..', '..', 'doc');
const doc1 = join(docdir, 'ok.md');
const doc2 = join(docdir, 'notok.md');

context('Watcher', () => {
    describe('#constructor', () => {
        it('generates watcher object with callbacks', () => {
            assert(
                new Watcher(
                    'dummypath',
                    function(a, b){ /* empty */ },
                    function(a){ /* empty */ }
                )
            );
        });
    });

    it('starts watching specified file automatically', done => {
        const watcher = new Watcher(
            doc1,
            function(kind: string, content: Object) {
                assert(kind === 'markdown');
                assert(content === doc1);
                done();
            },
            function(a) { /* do nothing */ }
        );
        assert(watcher.path === doc1);
    });

    it('starts watching specified directory automatically', done => {
        const watcher = new Watcher(
            docdir,
            function (kind, content){
                assert(kind === 'markdown');
                assert(content === doc2);
                done();
            },
            function(a) { /* do nothing */ }
        );
        assert(watcher.path === docdir);
        setTimeout(() => touch(doc2), 500);
    });


    describe('#changeWatchingDir()', () => {
        it('changes the watching dir', done => {
            const watcher = new Watcher(
                doc1,
                function(a, b){ /* empty */ },
                function(a){ /* empty */ }
            );
            let is_done = false;
            const done_once = () => {
                if (!is_done) {
                    done();
                    is_done = true;
                }
            };
            watcher.render = function(a, b) {
                assert(a === 'markdown');
                assert(watcher.path === doc2);
                done_once();
            };
            watcher.changeWatchingDir(doc2);
        });
    });
});
