'use strict';

function Linter(name) {
    if (name === 'markdownlint') {
        this.lint = this.markdownlint;
        this.lint_url = 'https://github.com/DavidAnson/markdownlint/blob/master/doc/Rules.md';
    } else if (name === 'mdast-lint') {
        this.lint = this.mdast_lint;
        this.lint_url = 'https://github.com/wooorm/mdast-lint/blob/master/doc/rules.md';
    } else if (name === 'none') {
        this.lint = function(f, c, p){};
        this.lint_url = '';
    } else {
        console.log("linter.js: Invalid linter name '" + name + "'");
        this.lint = function(f, c, p){};
        this.lint_url = '';
    }
}

Linter.prototype.markdownlint = function(filename, content, callback) {
    this.markdownlint = this.markdownlint || require('markdownlint');

    let options = {
        // TODO: Enable to specify lint configurations
        'strings' : {}
    };
    options.strings[filename] = content;

    this.markdownlint(options, function(err, result){
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
    this.mdast = this.mdast || require('mdast')().use(require('mdast-lint'));

    this.mdast.process(content, function(err, _, file){
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
