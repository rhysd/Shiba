import * as fs from 'fs';
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
import * as presetRecommended from 'remark-preset-lint-recommended';
import * as presetConsistent from 'remark-preset-lint-consistent';
import marker from './rehype_message_markers';
import schema from './sanitation_schema';

const rules = Object.assign({}, presetRecommended.plugins.lint, presetConsistent.plugins.lint);

export default class MarkdownProcessor {
    compiler: Unified.Processor;

    constructor() {
        this.compiler = unified({
            presets: ['lint-recommended'],
        }).use(
            parse
        ).use(
            rehype2react, {sanitize: schema}
        ).use(
            lint, rules
        ).use(
            emoji, {padSpaceAfter: true}
        ).use([
            slug,
            headings,
            github,
            toc,
            remark2rehype,
            marker,
        ]);
    }

    processFile(file: string): Promise<Unified.VFile> {
        return new Promise<string>((resolve, reject) => {
            fs.readFile(file, 'utf8', (err, doc) => {
                if (err) {
                    return reject(err);
                }
                resolve(doc);
            });
        }).then(doc => this.process(doc));
    }

    process(markdown: string): Promise<Unified.VFile> {
        return new Promise<Unified.VFile>((resolve, reject) => {
            this.compiler.process(markdown, (err, vfile) => {
                if (err) {
                    return reject(err);
                }
                resolve(vfile);
            });
        });
    }
}

