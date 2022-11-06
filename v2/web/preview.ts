import { unified } from 'unified';
import type { Plugin } from 'unified';
import type { Root as Hast, Text as HastText, Element as HastElement } from 'hast';
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
import type { SearchMatcher } from './ipc';
import * as log from './log';

// From https://github.com/rhysd/fast-json-clone
function cloneJson(x: any): any {
    if (typeof x !== 'object' || x === null) {
        return x;
    } else if (Array.isArray(x)) {
        return x.map(e => (typeof e !== 'object' || e === null ? e : cloneJson(e)));
    } else {
        const ret: { [k: string]: any } = {};
        for (const k in x) {
            const v = x[k];
            ret[k] = typeof v !== 'object' || v === null ? v : cloneJson(v);
        }
        return ret;
    }
}

// Allow `class` attribute in all HTML elements for highlight.js
defaultSchema.attributes!['*']!.push('className');

export type ReactElement = React.ReactElement<unknown>;
export interface PreviewContent {
    react: ReactElement;
    hast: Hast;
}

interface Matcher {
    findNextMatch(): [number, number] | null;
    setInput(input: string): void;
}

class CaseSensitiveMatcher implements Matcher {
    private readonly sep: string;
    private input = '';
    private index = 0;

    constructor(sep: string) {
        this.sep = sep;
    }

    setInput(input: string): void {
        this.input = input;
        this.index = 0;
    }

    findNextMatch(): [number, number] | null {
        const idx = this.input.indexOf(this.sep);

        if (idx < 0) {
            this.index += this.input.length;
            this.input = '';
            return null;
        }

        const start = this.index + idx;
        const end = start + this.sep.length;
        this.input = this.input.slice(end);
        this.index = end;
        return [start, end];
    }
}

class CaseInsensitiveMatcher extends CaseSensitiveMatcher implements Matcher {
    constructor(sep: string) {
        super(sep.toLowerCase());
    }

    override setInput(input: string): void {
        super.setInput(input.toLowerCase());
    }
}

const RE_UPPER_CASE = /[A-Z]/;
function selectMatcher(query: string, matcher: SearchMatcher): Matcher {
    switch (matcher) {
        case 'SmartCase':
            if (RE_UPPER_CASE.test(query)) {
                return new CaseSensitiveMatcher(query);
            } else {
                return new CaseInsensitiveMatcher(query);
            }
        case 'CaseSensitive':
            return new CaseSensitiveMatcher(query);
        case 'CaseInsensitive':
            return new CaseInsensitiveMatcher(query);
        default:
            log.error('Unknown search matcher:', matcher);
            return new CaseSensitiveMatcher(query); // fallback
    }
}

// TODO: Current implementation cannot search accross multiple Markdown elements.
// For example, document 'foo `bar`' is not hit when searching 'foo bar' since 'foo ' is a text and 'bar' is a inline code.

function highlight(matcher: Matcher, index: number | null, tree: Hast): void {
    function text(value: string, position?: Position): HastText {
        return {
            type: 'text',
            value,
            position,
        };
    }

    function span(s: string, current: boolean, position?: Position): HastElement {
        return {
            type: 'element',
            tagName: 'span',
            properties: {
                className: current ? 'search-text-current' : 'search-text',
            },
            children: [text(s, position)],
            position,
        };
    }

    function textToElem(node: any, children: Array<HastText | HastElement>): void {
        node.type = 'element';
        node.tagName = 'span';
        node.properties = {};
        node.children = children;
    }

    let count = 0;
    visit(tree, ['text'], node => {
        if (node.type !== 'text') {
            return;
        }

        const pos = node.position;
        const children: Array<HastText | HastElement> = [];
        const input = node.value;
        let lastEnd = 0;

        matcher.setInput(input);
        for (;;) {
            const range = matcher.findNextMatch();
            if (range === null) {
                if (children.length === 0) {
                    // No match was found. Skip this node
                    return SKIP;
                }
                if (input.length > 0) {
                    children.push(text(input.slice(lastEnd), pos));
                }
                break;
            }
            const [start, end] = range;
            if (start !== lastEnd) {
                children.push(text(input.slice(lastEnd, start), pos));
            }
            children.push(span(input.slice(start, end), index !== null && count === index, pos));
            lastEnd = end;
            count++;
        }

        textToElem(node, children);

        // Return SKIP to avoid infinite recursive calls due to generated <span> elements
        return SKIP;
    });
}

interface HighlightOptions {
    matcher: Matcher;
    index: number | null;
}

const highlightPlugin: Plugin<[HighlightOptions], Hast, Hast> = ({ matcher, index }) =>
    highlight.bind(this, matcher, index);

const RehypeReactConfig = { createElement, Fragment };

export async function parseMarkdown(content: string, query: string, config: SearchMatcher): Promise<PreviewContent> {
    let hast: Hast | null = null;
    const plugin: Plugin<[], Hast, Hast> = () => tree => {
        if (query) {
            hast = cloneJson(tree);
            highlight(selectMatcher(query, config), null, tree);
        } else {
            hast = tree;
        }
    };

    const compiler = unified()
        .use(remarkParse)
        .use(remarkFrontmatter)
        .use(remarkGfm)
        .use(remarkRehype)
        .use(rehypeHighlight, { plainText: ['txt', 'text'] })
        .use(rehypeSanitize, defaultSchema)
        .use(plugin)
        .use(rehypeReact, RehypeReactConfig);

    const file = await compiler.process(content);
    if (hast === null) {
        throw new Error('FATAL: HAST cache was not created');
    }

    return { react: file.result, hast };
}

export async function searchHast(
    tree: Hast,
    query: string,
    index: number | null,
    config: SearchMatcher,
): Promise<ReactElement> {
    if (query) {
        const matcher = selectMatcher(query, config);
        const options = { matcher, index };
        const transformer = unified().use(highlightPlugin, options).use(rehypeReact, RehypeReactConfig);
        const cloned = cloneJson(tree); // Compiler modifies the tree directly
        const transformed = await transformer.run(cloned);
        return transformer.stringify(transformed);
    } else {
        const transformer = unified().use(rehypeReact, RehypeReactConfig);
        return transformer.stringify(tree);
    }
}
