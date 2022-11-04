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

function highlight(searchText: string, tree: Hast): void {
    if (searchText.length === 0) {
        return;
    }

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

    visit(tree, ['text'], node => {
        if (node.type !== 'text') {
            return;
        }

        const split = node.value.split(searchText);
        if (split.length <= 1) {
            return;
        }
        const pos = node.position;

        const children: Array<HastText | HastElement> = [];
        if (split[0].length > 0) {
            children.push(text(split[0], pos));
        }

        const x = span(searchText, pos);
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

const highlightPlugin: Plugin<[string], Hast, Hast> = searchText => highlight.bind(this, searchText);

const RehypeReactConfig = { createElement, Fragment };

export async function parseMarkdown(content: string, searchText: string): Promise<PreviewContent> {
    let hast: Hast | null = null;
    const plugin: Plugin<[], Hast, Hast> = () => tree => {
        if (searchText) {
            hast = structuredClone(tree);
            highlight(searchText, tree);
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

export async function searchHast(tree: Hast, searchText: string): Promise<ReactElement> {
    if (searchText) {
        const transformer = unified().use(highlightPlugin, searchText).use(rehypeReact, RehypeReactConfig);
        const cloned = structuredClone(tree); // Compiler modifies the tree directly
        const transformed = await transformer.run(cloned);
        return transformer.stringify(transformed);
    } else {
        const transformer = unified().use(rehypeReact, RehypeReactConfig);
        return transformer.stringify(tree);
    }
}
