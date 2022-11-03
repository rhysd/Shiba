import { unified } from 'unified';
import type { Processor } from 'unified';
import type { Node } from 'unist';
import remarkParse from 'remark-parse';
import remarkFrontmatter from 'remark-frontmatter';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import rehypeHighlight from 'rehype-highlight';
import rehypeSanitize, { defaultSchema } from 'rehype-sanitize';
import rehypeReact from 'rehype-react';
import { createElement, Fragment } from 'react';
import * as log from './log';

defaultSchema.attributes!['*']!.push('className'); // Allow `class` attribute in all HTML elements

export type ContentCallback = (elem: React.ReactNode) => void;

export class Markdown {
    private compiler: Processor<Node, Node, Node, React.ReactElement>;
    private onContent: ContentCallback;

    constructor() {
        this.compiler = unified()
            .use(remarkParse)
            .use(remarkFrontmatter)
            .use(remarkGfm)
            .use(remarkRehype)
            .use(rehypeHighlight, { plainText: ['txt', 'text'] })
            .use(rehypeSanitize, defaultSchema)
            .use(rehypeReact, { createElement, Fragment });
        this.onContent = () => {};
    }

    registerCallback(callback: ContentCallback): void {
        this.onContent = callback;
        log.debug('Registered new content callback');
    }

    async parse(source: string) {
        const file = await this.compiler.process(source);
        this.onContent(file.result);
    }
}
