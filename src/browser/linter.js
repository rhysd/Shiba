'use strict';

let markdownlint = null;
let mdast = null;

function Linter(name) {
    if (this.name === 'markdownlint') {
        this.lint = this.markdownlint;
        this.lint_url = 'https://github.com/DavidAnson/markdownlint/blob/master/doc/Rules.md';
    } else if (this.name === 'mdast-lint') {
        this.lint = this.mdast_lint;
        this.lint_url = 'https://github.com/wooorm/mdast-lint/blob/master/doc/rules.md';
    } else if (this.name === 'none') {
        this.lint = function(f, c, p){};
        this.lint_url = '';
    } else {
        this.lint = this.mdast_lint;
        this.lint_url = 'https://github.com/wooorm/mdast-lint/blob/master/doc/rules.md';
    }
}

Linter.prototype.markdownlint = function(filename, content, callback) {
    if (!markdownlint) {
        markdownlint = require('markdownlint');
    }
    let options = {
        // TODO: Enable to specify lint configurations
        'strings' : {}
    };
    options.strings[filename] = content;

    markdownlint(options, function(err, result){
        if (err) {
            return [];
        }
        const is_space = /\s+/;
        const messages = result.toString()
                          .split("\n")
                          .map(function(msg){
                              const split = msg.split(is_space);
                              return {header: split[0], body: split[1]};
                      });
        callback(messages);
    });
};

Linter.prototype.mdast_lint = function(filename, content, callback) {
    if (!mdast) {
        mdast = require('mdast')().use(require('mdast-lint'));
    }

    mdast.process(content, function(err, _, file){
        if (err) {
            return [];
        }

        callback(
            file.messages.map(function(m){
                // Note:
                // Should I include m.ruleId to check the detail of message?
                // I don't include it now because message gets too long.
                return {
                    header: `${filename}:${m.line}:${m.column}:`,
                    body: m.reason
                };
            })
        );
    });
};

module.exports = Linter;
