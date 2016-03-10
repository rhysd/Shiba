import Linter from '../../browser/linter';

context('Linter', () => {
    describe('constructor', () => {
        it('generates linter object', () => {
            assert.isDefined(Linter);
            assert.isDefined(new Linter('markdownlint', {}));
        });

        it("accepts names 'markdownlint', 'remark-lint' and 'none'", () => {
            const lint_url_of = name => (new Linter(name, {})).lint_url;
            const lint_of = name => (new Linter(name, {})).lint;
            assert.ok(lint_url_of('markdownlint'));
            assert.isFunction(lint_of('markdownlint'));
            assert.ok(lint_url_of('remark-lint'));
            assert.isFunction(lint_of('remark-lint'));
            assert.notOk(lint_url_of('none'));
            assert.isFunction(lint_of('none'));
            assert.notOk(lint_url_of('unknown_linter_name'));
            assert.isFunction(lint_of('unknown_linter_name'));
        });
    });

    describe('markdownlint', () => {
        const linter = new Linter('markdownlint', {});

        it('lints markdown source', () => {
            linter.lint('foo.md', "## foo\n- bar\n  - poyo", msgs => assert.ok(msgs.length > 0));
            linter.lint('foo.md', "Foo\n===\n", msgs => assert.ok(msgs.length === 0));
        });
    });

    describe('remark-lint', () => {
        const linter = new Linter('remark-lint', {});

        it('lints markdown source', () => {
            linter.lint('foo.md', "## foo\n- bar\n  - poyo", msgs => assert.ok(msgs.length > 0));
            linter.lint('foo.md', "Foo\n===\n", msgs => assert.ok(msgs.length === 0));
        });
    });
});
