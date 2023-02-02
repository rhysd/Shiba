import * as React from 'react';
import hljs from 'highlight.js';

export interface Props {
    lang: string;
    code: string;
}

export const CodeBlock: React.FC<Props> = ({ lang, code }) => {
    const html = hljs.highlight(code, { language: lang }).value;
    return <code className={`language-${lang}`} dangerouslySetInnerHTML={{ __html: html }} />; // eslint-disable-line @typescript-eslint/naming-convention
};
