import {readFile} from 'fs';
import {ipcMain as ipc} from 'electron';
import * as markdownlint from 'markdownlint';
import * as remark from 'remark';
import remarkLintConsistent = require('remark-preset-lint-consistent');

interface RemarkFile {
    messages: {
        line: number;
        column: number;
        reason: string;
    }[];
}

export default class Linter {
    lint: (filename: string) => void;
    lint_url: string;
    remark: any;
    options: object;

    constructor(public sender: Electron.WebContents, name: string, options: object) {
        this.options = options || {};

        if (name === 'markdownlint') {
            this.lint = this.markdownlint;
            this.lint_url = 'https://github.com/DavidAnson/markdownlint/blob/master/doc/Rules.md';
        } else if (name === 'remark-lint' || name === 'mdast-lint') {
            this.lint = this.remark_lint;
            this.lint_url = 'https://github.com/wooorm/remark-lint/blob/master/doc/rules.md';
        } else if (name === 'none') {
            this.lint = function(_) { /* do nothing */ };
            this.lint_url = '';
        } else {
            console.log(`linter.js: Invalid linter name '${name}'`);
            this.lint = function(_) { /* do nothing */ };
            this.lint_url = '';
        }

        ipc.on('shiba:request-lint-rule-url', () => {
            this.sender.send('shiba:return-lint-rule-url', this.lint_url);
        });
    }

    sendResult(messages: LinterMessage[]) {
        this.sender.send('shiba:notify-linter-result', messages);
    }

    markdownlint(filename: string) {
        readFile(filename, 'utf8', (read_err: Error, content: string) => {
            if (read_err) {
                console.error(read_err);
                return;
            }

            const opts = {
                strings: {
                    [filename]: content,
                },
                config: this.options,
            };

            markdownlint(opts, (err: Error, result: any) => {
                if (err) {
                    return;
                }
                const is_space = /\s+/;
                const messages = result.toString()
                                .split('\n')
                                .filter((msg: string) => msg !== '')
                                .map(function(msg: string): LinterMessage {
                                    const m = msg.match(is_space);
                                    if (!m) {
                                        return {header: '', body: msg};
                                    }

                                    return {
                                        header: msg.slice(0, m.index),
                                        body: msg.slice(m.index),
                                    };
                                });
                this.sendResult(messages);
            });
        });
    }

    remark_lint(filename: string) {
        readFile(filename, 'utf8', (read_err: Error, content: string) => {
            if (read_err) {
                console.error(read_err);
                return;
            }

            this.remark = this.remark ||
                remark().use(remarkLintConsistent);

            this.remark.process(content, (err: NodeJS.ErrnoException, file: RemarkFile) => {
                if (err) {
                    console.log('Lint failed: ', err.stack);
                    return;
                }

                this.sendResult(
                    file.messages.map(function(m): LinterMessage {
                        // Note:
                        // Should I include m.ruleId to check the detail of message?
                        // I don't include it now because message gets too long.
                        return {
                            header: `line:${m.line}, col:${m.column}`,
                            body: m.reason,
                            line: m.line,
                            column: m.column,
                        };
                    }),
                );
            });
        });
    }

}

