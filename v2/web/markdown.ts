import { unified } from 'unified';
import type { Processor, Plugin } from 'unified';
import type { Root as Hast, Text as HastText, Element as HastElement } from 'hast';
import type { Root as Mdast } from 'mdast';
import type { Position } from 'unist';
import remarkParse from 'remark-parse';
import remarkFrontmatter from 'remark-frontmatter';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import rehypeHighlight from 'rehype-highlight';
import rehypeSanitize, { defaultSchema } from 'rehype-sanitize';
import rehypeReact from 'rehype-react';
import { visit, SKIP } from 'unist-util-visit';
import { createElement, Fragment } from 'react';
import * as log from './log';

// WKWebView does not have `structuredClone` though Safari has: https://caniuse.com/mdn-api_structuredclone
if (typeof structuredClone === 'undefined') {
    // Using `JSON.parse` and `JSON.stringify` is about 3.8x faster than @ungap/structured-clone though
    // the parameter is limited to JSON-compatible value.
    (window as any).structuredClone = (x: unknown) => JSON.parse(JSON.stringify(x));
}

defaultSchema.attributes!['*']!.push('className'); // Allow `class` attribute in all HTML elements

type ReactElement = React.ReactElement<unknown>;
export type ContentCallback = (elem: ReactElement) => void;

const RehypeReactConfig = { createElement, Fragment };

class SearchTextHighlight {
    private text: string;

    constructor() {
        this.text = '';
    }

    transform(tree: Hast): void {
        function text(value: string, position?: Position): HastText {
            return {
                type: 'text',
                value,
                position,
            };
        }

        function span(s: string, position?: Position): HastElement {
            return {
                type: 'element',
                tagName: 'span',
                properties: {
                    className: 'search-text',
                },
                children: [text(s, position)],
                position,
            };
        }

        function textToElem(node: any, children: Array<HastText | HastElement>) {
            node.type = 'element';
            node.tagName = 'span';
            node.properties = {};
            node.children = children;
        }

        if (this.text.length === 0) {
            return;
        }

        visit(tree, ['text'], node => {
            if (node.type !== 'text') {
                return;
            }

            const split = node.value.split(this.text);
            if (split.length <= 1) {
                return;
            }
            const pos = node.position;

            const children: Array<HastText | HastElement> = [];
            if (split[0].length > 0) {
                children.push(text(split[0], pos));
            }

            const x = span(this.text, pos);
            for (const s of split.slice(1)) {
                children.push(x);
                if (s.length > 0) {
                    children.push(text(s, pos));
                }
            }

            textToElem(node, children);

            // Return SKIP to avoid infinite recursive calls due to generated <span> elements
            return SKIP;
        });
    }

    createPlugin(): Plugin<[], Hast, Hast> {
        return () => this.transform.bind(this);
    }

    setText(text: string): boolean {
        if (this.text === text) {
            return false;
        } else {
            this.text = text;
            return true;
        }
    }

    searching(): boolean {
        return this.text.length > 0;
    }
}

class TextSearch {
    private tree: Hast | null;
    private processor: Processor<void, Hast, Hast, ReactElement>;
    private resetter: Processor<void, Hast, Hast, ReactElement>;
    private highlighter: SearchTextHighlight;

    constructor() {
        this.tree = null;
        this.highlighter = new SearchTextHighlight();
        this.processor = unified().use(this.highlighter.createPlugin()).use(rehypeReact, RehypeReactConfig);
        this.resetter = unified().use(rehypeReact, RehypeReactConfig);
    }

    createPlugin(): Plugin<[], Hast, Hast> {
        return () => tree => {
            if (this.highlighter.searching()) {
                this.tree = structuredClone(tree);
                this.highlighter.transform(tree);
            } else {
                this.tree = tree;
            }
        };
    }

    async search(text: string): Promise<ReactElement | null> {
        if (this.tree === null) {
            return null;
        }
        if (!this.highlighter.setText(text)) {
            return null;
        }
        const tree = structuredClone(this.tree);
        const transformed = await this.processor.run(tree);
        return this.processor.stringify(transformed);
    }

    async reset(): Promise<ReactElement | null> {
        if (!this.highlighter.setText('')) {
            return null;
        }
        if (this.tree === null) {
            return null;
        }
        const transformed = await this.resetter.run(this.tree);
        return this.resetter.stringify(transformed);
    }
}

export class Markdown {
    private markdownProcessor: Processor<Mdast, Hast, Hast, ReactElement>;
    private render: ContentCallback;
    private searcher: TextSearch;

    constructor() {
        this.searcher = new TextSearch();
        this.markdownProcessor = unified()
            .use(remarkParse)
            .use(remarkFrontmatter)
            .use(remarkGfm)
            .use(remarkRehype)
            .use(rehypeHighlight, { plainText: ['txt', 'text'] })
            .use(rehypeSanitize, defaultSchema)
            .use(this.searcher.createPlugin())
            .use(rehypeReact, RehypeReactConfig);
        this.render = () => {};
    }

    registerCallback(callback: ContentCallback): void {
        this.render = callback;
        log.debug('Registered new content callback');
    }

    async parse(source: string): Promise<void> {
        const file = await this.markdownProcessor.process(source);
        this.render(file.result);
    }

    async search(text: string | null): Promise<void> {
        if (text) {
            const highlighted = await this.searcher.search(text);
            if (highlighted !== null) {
                this.render(highlighted);
            }
        } else {
            const reset = await this.searcher.reset();
            if (reset !== null) {
                this.render(reset);
            }
        }
    }
}
