'use strict';

let markdownlint = require('markdownlint');
// let mdast = require('mdast');
// let lint = require('mdast-lint');

function Linter(name) {
    if (this.name === 'markdownlint') {
        this.lint = this.markdownlint;
        this.lint_url = 'https://github.com/DavidAnson/markdownlint/blob/master/doc/Rules.md';
    } else if (this.name === 'mdast-lint') {
        this.lint = this.mdast_lint;
        this.lint_url = 'https://github.com/wooorm/mdast-lint/blob/master/doc/rules.md';
    } else {
        this.lint = this.markdownlint;
        this.lint_url = 'https://github.com/DavidAnson/markdownlint/blob/master/doc/Rules.md';
    }
}

Linter.prototype.markdownlint = function(filename, content, callback) {
    let options = {
        // TODO: Enable to specify lint configurations
        'strings' : {}
    };
    options.strings[filename] = content;

    markdownlint(options, function(err, result){
        if (!err) {
            callback(result.toString());
        }
    });
};

Linter.prototype.mdast_lint = function(filename, content, callback) {
    callback('');
};

module.exports = Linter;
