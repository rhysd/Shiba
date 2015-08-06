import Linter from '../../src/browser/linter';

context('Linter', () => {
    describe('constructor', () => {
        it('generates linter object', () => {
            assert.isDefined(Linter);
            assert.isDefined(new Linter('markdownlint', {}));
        });

        it("accepts names 'markdownlint', 'mdast-lint' and 'none'", () => {
            const lint_url_of = name => (new Linter(name, {})).lint_url;
            const lint_of = name => (new Linter(name, {})).lint;
            assert.ok(lint_url_of('markdownlint'));
            assert.isFunction(lint_of('markdownlint'));
            assert.ok(lint_url_of('mdast-lint'));
            assert.isFunction(lint_of('mdast-lint'));
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

    describe('mdast-lint', () => {
        const linter = new Linter('mdast-lint', {});

        it('lints markdown source', () => {
            linter.lint('foo.md', "## foo\n- bar\n  - poyo", msgs => assert.ok(msgs.length > 0));
            linter.lint('foo.md', "Foo\n===\n", msgs => assert.ok(msgs.length === 0));
        });
    });
});
