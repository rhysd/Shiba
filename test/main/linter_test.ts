/// <reference path="lib.d.ts" />

import Linter from '../../browser/linter';
import * as assert from 'power-assert';
import {join} from 'path';

// Compiled into './test/main/' directory
const ok_doc = join(__dirname, '..', '..', 'doc', 'ok.md');
const not_ok_doc = join(__dirname, '..', '..', 'doc', 'notok.md');

context('Linter', () => {
    describe('constructor', () => {
        it('accepts default linter name', () => {
            assert(new Linter('markdownlint', {}));
        });

        it("accepts names 'markdownlint', 'remark-lint' and 'none'", () => {
            assert(new Linter('markdownlint', {}).lint_url);
            assert(new Linter('markdownlint', {}).lint);
            assert(new Linter('remark-lint', {}).lint_url);
            assert(new Linter('remark-lint', {}).lint);
            assert(!new Linter('none', {}).lint_url);
            assert(new Linter('none', {}).lint);
            assert(!new Linter('unknown_linter_name', {}).lint_url);
            assert(new Linter('unknown_linter_name', {}).lint);
        });
    });

    describe('markdownlint', () => {
        const linter = new Linter('markdownlint', {});

        it('lints markdown source', () => {
            linter.lint(
                not_ok_doc,
                (msgs: LinterMessage[]) => {
                    assert(msgs.length > 0);
                }
            );
            linter.lint(
                ok_doc,
                (msgs: LinterMessage[]) => {
                    assert(msgs.length === 0);
                }
            );
        });
    });

    describe('remark-lint', () => {
        const linter = new Linter('remark-lint', {});

        it('lints markdown source', () => {
            linter.lint(
                not_ok_doc,
                (msgs: LinterMessage[]) => {
                    assert(msgs.length > 0);
                }
            );
            linter.lint(
                ok_doc,
                (msgs: LinterMessage[]) => {
                    assert(msgs.length === 0);
                }
            );
        });
    });

    describe('none', () => {
        const linter = new Linter('none', {});

        it('does nothing and never fires callback', () => {
            linter.lint(not_ok_doc, (_: any) => { throw new Error('Never thrown'); });
        });
    });

    describe('invalid linter name', () => {
        const linter = new Linter('invalid-linter-name', {});

        it('does nothing and never fires callback', () => {
            linter.lint(not_ok_doc, (_: any) => { throw new Error('Never thrown'); });
        });
    });

    it('doesn not crash on invalid file path', () => {
        const linter = new Linter('remark-lint', {});
        linter.lint('path/to/not/existing/file', (_: any) => { /* do nothing */ });
        linter.lint('', (a: any) => { throw new Error('Never thrown'); });
    });
});
