import Watcher = require('../../src/browser/watcher');
import {join} from 'path';

context('Watcher', () => {
    describe('#constructor', () => {
        it('is not undefined', () => assert.ok(Watcher));
        it('generates watcher object', () => assert.ok(new Watcher('dummypath', function(a, b){}, function(a){})));
    });

    it('starts watching specified file automatically', done => {
        const file = join(process.cwd(), 'README.md');
        let watcher = new Watcher(file, function(kind, content){
            assert.strictEqual(kind, 'markdown');
            assert.strictEqual(watcher.path, file);
            done();
        }, function(a){});
    });

    it('starts to lint specified file automatically', done => {
        const file = join(process.cwd(), 'README.md');
        let watcher = new Watcher(file, function(a, b){}, function(msgs){ done(); });
    });

    describe('#changeWatchingDir()', () => {
        it('changes the watching dir', done => {
            const file = join(process.cwd(), 'README.md');
            let watcher = new Watcher(file, function(a, b){}, function(a){});
            const next_file = join(process.cwd(), 'docs', 'usage.md');
            let is_done = false;
            const done_once = () => {
                if (!is_done) {
                    done();
                    is_done = true;
                }
            };
            watcher.render = function(a, b) {
                assert.strictEqual(a, 'markdown');
                assert.strictEqual(watcher.path, next_file);
                done_once();
            };
            watcher.changeWatchingDir(next_file);
        });

        it('changes the linting dir', done => {
            const file = join(process.cwd(), 'README.md');
            let watcher = new Watcher(file, function(a, b){}, function(a){});
            const next_file = join(process.cwd(), 'docs', 'usage.md');
            let is_done = false;
            const done_once = () => {
                if (!is_done) {
                    done();
                    is_done = true;
                }
            };
            watcher.renderLintResult = function(m){ done_once(); };
            watcher.changeWatchingDir(next_file);
        });
    });

    describe('#getLintRuleURL', () => {
        it('returns lint URL', () => {
            const url = (new Watcher('dummy', function(a, b){}, function(a){})).getLintRuleURL();
            assert.ok(url);
            assert.match(url, /^https?:\/\//);
        });
    });
});
