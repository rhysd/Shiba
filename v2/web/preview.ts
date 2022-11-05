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

// WKWebView does not have `structuredClone` though Safari has: https://caniuse.com/mdn-api_structuredclone
if (typeof structuredClone === 'undefined') {
    // Using `JSON.parse` and `JSON.stringify` is about 3.8x faster than @ungap/structured-clone though
    // the parameter is limited to JSON-compatible value.
    (window as any).structuredClone = (x: unknown) => JSON.parse(JSON.stringify(x));
}

// Allow `class` attribute in all HTML elements for highlight.js
defaultSchema.attributes!['*']!.push('className');

export type ReactElement = React.ReactElement<unknown>;
export interface PreviewContent {
    react: ReactElement;
    hast: Hast;
}

function highlight(query: string, index: number | null, tree: Hast): void {
    if (query.length === 0) {
        return;
    }

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

    function textToElem(node: any, children: Array<HastText | HastElement>) {
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

        const split = node.value.split(query);
        if (split.length <= 1) {
            return;
        }
        const pos = node.position;

        const children: Array<HastText | HastElement> = [];
        if (split[0].length > 0) {
            children.push(text(split[0], pos));
        }

        for (const s of split.slice(1)) {
            children.push(span(query, index !== null && count === index, pos));
            count++;
            if (s.length > 0) {
                children.push(text(s, pos));
            }
        }

        textToElem(node, children);

        // Return SKIP to avoid infinite recursive calls due to generated <span> elements
        return SKIP;
    });
}

interface HighlightOptions {
    query: string;
    index: number | null;
}

const highlightPlugin: Plugin<[HighlightOptions], Hast, Hast> = ({ query, index }) =>
    highlight.bind(this, query, index);

const RehypeReactConfig = { createElement, Fragment };

export async function parseMarkdown(content: string, query: string): Promise<PreviewContent> {
    let hast: Hast | null = null;
    const plugin: Plugin<[], Hast, Hast> = () => tree => {
        if (query) {
            hast = structuredClone(tree);
            highlight(query, null, tree);
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

export async function searchHast(tree: Hast, query: string, index: number | null): Promise<ReactElement> {
    if (query) {
        const options = { query, index };
        const transformer = unified().use(highlightPlugin, options).use(rehypeReact, RehypeReactConfig);
        const cloned = structuredClone(tree); // Compiler modifies the tree directly
        const transformed = await transformer.run(cloned);
        return transformer.stringify(transformed);
    } else {
        const transformer = unified().use(rehypeReact, RehypeReactConfig);
        return transformer.stringify(tree);
    }
}
