var assert = require('chai').assert;
var Linter = require('../../build/src/browser/linter').default;

describe('Linter', function() {
    describe('constructor', function() {
        it('generates linter object', function() {
            assert.isDefined(Linter);
            assert.isDefined(new Linter('markdownlint', {}));
        });

        it("accepts names 'markdownlint', 'mdast-lint' and 'none'", function() {
            var lint_url_of = function(name){ return (new Linter(name, {})).lint_url; };
            var lint_of = function(name){ return (new Linter(name, {})).lint; };
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

    describe('markdownlint', function() {
        var linter = new Linter('markdownlint', {});

        it('lints markdown source', function() {
            linter.lint('foo.md', "## foo\n- bar\n  - poyo", function(msgs) {
                assert.ok(msgs.length > 0);
            });
            linter.lint('foo.md', "Foo\n===\n", function(msgs) {
                assert.ok(msgs.length === 0);
            });
        });
    });

    describe('mdast-lint', function() {
        var linter = new Linter('mdast-lint', {});

        it('lints markdown source', function() {
            linter.lint('foo.md', "## foo\n- bar\n  - poyo", function(msgs) {
                assert.ok(msgs.length > 0);
            });
            linter.lint('foo.md', "Foo\n===\n", function(msgs) {
                assert.ok(msgs.length === 0);
            });
        });
    });
});
