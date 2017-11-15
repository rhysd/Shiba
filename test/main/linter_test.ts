/// <reference path="lib.d.ts" />

import Linter from '../../browser/linter';
import DummyWebContents from './dummy_webcontents';
import * as assert from 'power-assert';
import {join} from 'path';
import {ipcMain as ipc} from 'electron';

// Compiled into './test/main/' directory
const ok_doc = join(__dirname, '..', '..', 'doc', 'ok.md');
const not_ok_doc = join(__dirname, '..', '..', 'doc', 'notok.md');
const not_ok_doc_remark = join(__dirname, '..', '..', 'doc', 'notok_remark.md');

context('Linter', () => {
    describe('constructor', () => {
        const dummy_contents = new DummyWebContents() as any;

        it('accepts default linter name', () => {
            assert(new Linter(dummy_contents, 'markdownlint', {}));
        });

        it("accepts names 'markdownlint', 'remark-lint' and 'none'", () => {
            assert(new Linter(dummy_contents, 'markdownlint', {}).lint_url);
            assert(new Linter(dummy_contents, 'markdownlint', {}).lint);
            assert(new Linter(dummy_contents, 'remark-lint', {}).lint_url);
            assert(new Linter(dummy_contents, 'remark-lint', {}).lint);
            assert(!new Linter(dummy_contents, 'none', {}).lint_url);
            assert(new Linter(dummy_contents, 'none', {}).lint);
            assert(!new Linter(dummy_contents, 'unknown_linter_name', {}).lint_url);
            assert(new Linter(dummy_contents, 'unknown_linter_name', {}).lint);
        });
    });

    describe('markdownlint', () => {
        it('lints markdown source', done => {
            const c = new DummyWebContents() as any;
            const linter = new Linter(c, 'markdownlint', {});
            c.once(
                'shiba:notify-linter-result',
                (_: any, msgs: LinterMessage[]) => {
                    assert(msgs.length > 0);
                    done();
                },
            );
            linter.lint(not_ok_doc);
        });

        it('lints markdown source', done => {
            const c = new DummyWebContents() as any;
            const linter = new Linter(c, 'markdownlint', {});
            c.once(
                'shiba:notify-linter-result',
                (_: any, msgs: LinterMessage[]) => {
                    assert(msgs.length === 0);
                    done();
                },
            );
            linter.lint(ok_doc);
        });
    });

    describe('remark-lint', () => {
        it('reports some errors', done => {
            const c = new DummyWebContents() as any;
            const linter = new Linter(c, 'remark-lint', {});
            c.once(
                'shiba:notify-linter-result',
                (_: any, msgs: LinterMessage[]) => {
                    assert(msgs.length > 0);
                    done();
                },
            );
            linter.lint(not_ok_doc_remark);
        });

        it('reports no error for good doc', done => {
            const c = new DummyWebContents() as any;
            const linter = new Linter(c, 'remark-lint', {});
            c.once(
                'shiba:notify-linter-result',
                (_: any, msgs: LinterMessage[]) => {
                    assert(msgs.length === 0);
                    done();
                },
            );
            linter.lint(ok_doc);
        });

        it('refers "plugins" entry of config', done => {
            const c = new DummyWebContents() as any;
            const linter = new Linter(c, 'remark-lint', {plugins: []});
            c.once(
                'shiba:notify-linter-result',
                (_: any, msgs: LinterMessage[]) => {
                    // It should not raise an error because no preset is specified
                    assert(msgs.length === 0);
                    done();
                },
            );
            linter.lint(not_ok_doc_remark);
        });
    });

    describe('none', () => {
        it('lints markdown source', done => {
            const c = new DummyWebContents() as any;
            const linter = new Linter(c, 'remark-lint', {});
            c.once(
                'shiba:notify-linter-result',
                (_: any, msgs: LinterMessage[]) => {
                    assert(msgs.length === 0);
                    done();
                },
            );
            linter.lint(ok_doc);
        });
    });

    describe('invalid linter name', () => {
        it('does nothing and never fires callback', () => {
            const c = new DummyWebContents() as any;
            const linter = new Linter(c, 'invalid-linter-name', {});
            linter.lint(not_ok_doc);
        });
    });

    it('does not crash on invalid file path', () => {
        const c = new DummyWebContents() as any;
        const linter = new Linter(c, 'remark-lint', {});
        linter.lint('path/to/not/existing/file');
        linter.lint('');
    });

    it('returns linter\'s rules URL via IPC', done => {
        const c = new DummyWebContents() as any;
        new Linter(c, 'remark-lint', {});
        c.once('shiba:return-lint-rule-url', (_: any, url: string) => {
            assert.equal(url.indexOf('http'), 0);
            done();
        });
        ipc.emit('shiba:request-lint-rule-url', {});
    });
});
