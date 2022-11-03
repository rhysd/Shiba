import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkFrontmatter from 'remark-frontmatter';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import rehypeHighlight from 'rehype-highlight';
import rehypeSanitize, { defaultSchema } from 'rehype-sanitize';
import rehypeReact from 'rehype-react';
import { createElement, Fragment } from 'react';

defaultSchema.attributes!['*']!.push('className'); // Allow `class` attribute in all HTML elements

const remark = unified()
    .use(remarkParse)
    .use(remarkFrontmatter)
    .use(remarkGfm)
    .use(remarkRehype)
    .use(rehypeHighlight, { plainText: ['txt', 'text'] })
    .use(rehypeSanitize, defaultSchema)
    .use(rehypeReact, { createElement, Fragment });

export async function parseMarkdown(source: string): Promise<React.ReactNode> {
    const file = await remark.process(source);
    return file.result;
}
