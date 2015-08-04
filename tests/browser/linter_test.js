var Linter = require('../../build/src/browser/linter').default;

describe('Linter', function() {
    describe('constructor', function(){
        it('generates linter object', function() {
            (new Linter('markdownlint', {})).should.not.equal(undefined);
        });

        it("accepts names 'markdownlint', 'mdast-lint' and 'none'", function() {
            (new Linter('markdownlint', {})).lint_url.should.not.equal('');
            (new Linter('mdast-lint', {})).lint_url.should.not.equal('');
            (new Linter('none', {})).lint_url.should.equal('');
            (new Linter('unknown_linter_name', {})).lint_url.should.equal('');
        })
    });
});
