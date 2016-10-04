import * as fs from 'fs';
import * as React from 'react';
import * as unified from 'unified';
import * as parse from 'remark-parse';
import * as toc from 'remark-toc';
import * as slug from 'remark-slug';
import * as headings from 'remark-autolink-headings';
import * as github from 'remark-github';
import * as lint from 'remark-lint';
import * as remark2rehype from 'remark-rehype';
import * as rehype2react from 'rehype-react';
import * as emoji from 'remark-emoji';

/*
function isUnistParent(node: Unist.Node): node is Unist.Parent {
    return 'children' in node;
}

function test() {
    function visit(node: Hast.HastNode) {
        if (isUnistParent(node)) {
            for (const c of node.children) {
                visit(c);
            }
        }
        if (node.type === 'text') {
            node.value = 'Hello, from transformer';
        }
    }
    return visit;
}
*/

export default class MarkdownProcessor {
    compiler: Unified.Processor;

    constructor() {
        this.compiler = unified({
            presets: ['lint-recommended'],
        }).use([
            parse,
            rehype2react,
        ]).use(
            lint, {firstHeadingLevel: true}
        ).use(
            emoji, {padSpaceAfter: true}
        ).use([
            slug,
            headings,
            github,
            toc,
            remark2rehype,
        ]);
    }

    processFile(file: string): Promise<React.ReactElement<any>> {
        return new Promise<string>((resolve, reject) => {
            fs.readFile(file, 'utf8', (err, doc) => {
                if (err) {
                    return reject(err);
                }
                resolve(doc);
            });
        }).then(doc => this.process(doc));
    }

    process(markdown: string): Promise<React.ReactElement<any>> {
        return new Promise<React.ReactElement<any>>((resolve, reject) => {
            this.compiler.process(markdown, (err, vfile) => {
                if (err) {
                    return reject(err);
                }
                resolve(vfile.contents);
            });
        });
    }
}

