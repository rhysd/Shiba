'use strict';

function Linter(name, options) {
    this.options = options || {};

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

    let opts = {
        strings: {},
        config: this.options
    };
    opts.strings[filename] = content;

    this.markdownlint(opts, function(err, result){
        if (err) {
            return [];
        }
        const is_space = /\s+/;
        const messages = result.toString()
                          .split("\n")
                          .map(function(msg){
                              const m = msg.match(is_space);
                              if (!m) {
                                  return {header: '', body: msg};
                              }

                              return {
                                  header: msg.slice(0, m.index),
                                  body: msg.slice(m.index)
                              };
                      });
        callback(messages);
    });
};

Linter.prototype.mdast_lint = function(filename, content, callback) {
    this.mdast = this.mdast || require('mdast')().use(require('mdast-lint'), this.options);

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
