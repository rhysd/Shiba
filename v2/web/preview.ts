import { unified } from 'unified';
import type { Plugin } from 'unified';
import type { Root as Hast, Text as HastText, Element as HastElement } from 'hast';
import type { Position } from 'unist';
import remarkParse from 'remark-parse';
import remarkFrontmatter from 'remark-frontmatter';
import remarkGfm from 'remark-gfm';
import remarkGemoji from 'remark-gemoji';
import remarkRehype from 'remark-rehype';
import rehypeHighlight from 'rehype-highlight';
import rehypeSanitize, { defaultSchema } from 'rehype-sanitize';
import rehypeReact from 'rehype-react';
import rehypeRaw from 'rehype-raw';
import { visit, SKIP, EXIT } from 'unist-util-visit';
import { createElement, Fragment } from 'react';
import type { SearchMatcher } from './ipc';
import * as log from './log';
import { Link } from './components/Link';

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

function textToElem(node: any, children: Array<HastText | HastElement>): HastElement {
    node.type = 'element';
    node.tagName = 'span';
    node.properties = {};
    node.children = children;
    return node;
}

// Allow `class` attribute in all HTML elements for highlight.js
defaultSchema.attributes!['*']!.push('className');
// If this is set prefix 'user-content-' is set twice
defaultSchema.clobber = [];

export type ReactElement = React.ReactElement<unknown>;
export interface PreviewContent {
    react: ReactElement;
    hast: Hast;
}

interface Matcher {
    findNextMatch(): [number, number] | null;
    setInput(input: string): void;
}

class NullMatcher implements Matcher {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    setInput(_: string): void {
        // Do nothing
    }

    findNextMatch(): [number, number] | null {
        return null;
    }
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

class CaseSensitiveRegexMatcher implements Matcher {
    private readonly sep: RegExp;
    private input = '';
    private index = 0;

    constructor(sep: string) {
        this.sep = new RegExp(sep);
    }

    setInput(input: string): void {
        this.input = input;
        this.index = 0;
    }

    findNextMatch(): [number, number] | null {
        const m = this.input.match(this.sep);
        if (m === null) {
            this.index += this.input.length;
            this.input = '';
            return null;
        }

        const start = this.index + m.index!;
        const end = start + m[0].length;
        this.input = this.input.slice(end);
        this.index = end;
        return [start, end];
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
        case 'CaseSensitiveRegex':
            try {
                return new CaseSensitiveRegexMatcher(query);
            } catch (err) {
                log.debug('Could not create regex matcher:', err);
                return new NullMatcher();
            }
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

interface ChangeMarkerOptions {
    offset: number | null;
}
const changeMarkerPlugin: Plugin<[ChangeMarkerOptions], Hast, Hast> = ({ offset }) => {
    return tree => {
        if (offset === null) {
            return;
        }

        // Note: `let modified: HastElement | HastText | null = ...` does not work
        let modified = null as HastElement | HastText | null;

        visit(tree, node => {
            // Note: `start` and `end` may be `null` due to unknown reason
            const start = node.position?.start?.offset;
            const end = node.position?.end?.offset;
            if (start === undefined || end === undefined || offset < start || end < offset) {
                return;
            }

            switch (node.type) {
                case 'text': {
                    modified = node;
                    return EXIT;
                }
                case 'element': {
                    modified = node;
                    return;
                }
                default:
                    return;
            }
        });

        if (modified !== null) {
            switch (modified.type) {
                case 'text': {
                    const text = { ...modified };
                    const elem = textToElem(modified, [text]);
                    elem.properties = {
                        className: 'last-modified-marker',
                    };
                    log.debug('Last modified element:', elem);
                    break;
                }
                case 'element': {
                    modified.properties ??= {};
                    if ('className' in modified.properties) {
                        modified.properties['className'] += ' last-modified-marker';
                    } else {
                        modified.properties['className'] = 'last-modified-marker';
                    }
                    break;
                }
                default:
                    break;
            }
        }

        log.debug('Last modified node for offset', offset, modified);
    };
};

const RehypeReactConfig = {
    createElement,
    Fragment,
    components: {
        a: Link,
    },
};

export async function parseMarkdown(
    content: string,
    query: string,
    config: SearchMatcher,
    changeOffset: number | null,
): Promise<PreviewContent> {
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
        .use(remarkGemoji)
        .use(remarkRehype, { allowDangerousHtml: true })
        .use(rehypeRaw)
        .use(rehypeHighlight, { plainText: ['txt', 'text'], ignoreMissing: true })
        .use(rehypeSanitize, defaultSchema)
        .use(plugin)
        .use(changeMarkerPlugin, { offset: changeOffset })
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
