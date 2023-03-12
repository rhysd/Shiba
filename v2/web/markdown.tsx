import * as React from 'react';
import type { ReactNode, ReactElement } from 'react';
import hljs from 'highlight.js';
import Mathjax from 'react-mathjax-component';
import mermaid from 'mermaid';
import type { RenderTreeElem, RenderTreeFootNoteDef, RenderTreeTableAlign, RenderTreeCodeFence } from './ipc';
import type { Theme } from './reducer';
import * as log from './log';
import { Mermaid } from './components/Mermaid';

class FenceRenderer {
    private readonly theme: Theme;
    private static mermaidInitialized = false;
    private mermaidId = 0;

    constructor(theme: Theme) {
        this.theme = theme;
    }

    private initMermaid(): void {
        if (FenceRenderer.mermaidInitialized) {
            return;
        }
        const theme = this.theme === 'light' ? 'default' : 'dark';
        mermaid.initialize({ startOnLoad: false, theme });
        log.debug('Initialized mermaid renderer', theme);
        FenceRenderer.mermaidInitialized = true;
    }

    private async renderMermaid(content: string, key: number | undefined): Promise<ReactElement> {
        this.initMermaid();
        const id = this.mermaidId++;
        const { svg, bindFunctions } = await mermaid.render(`graph-${id}`, content);
        return <Mermaid svg={svg} bindFn={bindFunctions} key={key} />;
    }

    private renderHljs(code: string, lang: string, key: number | undefined): ReactElement {
        const html = hljs.highlight(code, { language: lang }).value;
        return <code className={`language-${lang}`} dangerouslySetInnerHTML={{ __html: html }} key={key} />; // eslint-disable-line @typescript-eslint/naming-convention
    }

    async render(elem: RenderTreeCodeFence, key?: number): Promise<[ReactElement, boolean] | null> {
        if (!elem.lang) {
            return null;
        }

        if (hljs.getLanguage(elem.lang)) {
            const text = childrenText(elem.c);
            if (text === null) {
                return null;
            }
            const [content, modified] = text;
            const rendered = this.renderHljs(content, elem.lang, key);
            return [rendered, modified];
        }

        if (elem.lang === 'mermaid') {
            const text = childrenText(elem.c);
            if (text === null) {
                return null;
            }
            const [content, modified] = text;
            const rendered = await this.renderMermaid(content, key);
            return [rendered, modified];
        }

        if (elem.lang === 'math') {
            const text = childrenText(elem.c);
            if (text === null) {
                return null;
            }
            const [content, modified] = text;
            return [
                <div className="code-fence-math" key={key}>
                    <Mathjax expr={content} />
                </div>,
                modified,
            ];
        }

        return null;
    }
}

const FOOTNOTE_BACKREF_STYLE: React.CSSProperties = {
    fontFamily: 'monospace',
    fontSize: '1.25em',
    margin: '0 0.25em',
};

export interface MarkdownReactTree {
    root: ReactNode;
    lastModified: React.RefObject<HTMLSpanElement> | null;
    matchCount: number;
}

function rawText(elem: RenderTreeElem): string {
    if (typeof elem === 'string') {
        return elem;
    }
    if ('c' in elem) {
        return elem.c.map(rawText).join('');
    }
    return '';
}

function childrenText(children: RenderTreeElem[]): null | [string, boolean] {
    let modified = false;
    let content = '';
    for (const child of children) {
        if (typeof child === 'string') {
            content += child;
        } else if (child.t === 'modified') {
            modified = true;
        } else {
            return null; // Reaches here when search highlight is included
        }
    }

    return [content, modified];
}

function isReactElement(node: ReactNode): node is ReactElement {
    return node !== null && typeof node === 'object' && '$$typeof' in node;
}

function lastElementOf(nodes: ReactNode[]): ReactElement | null {
    if (nodes.length === 0) {
        return null;
    }
    const last = nodes[nodes.length - 1];
    return isReactElement(last) ? last : null;
}

interface TableState {
    aligns: RenderTreeTableAlign[];
    index: number;
}

function tableAlignStyle({ aligns, index }: TableState): React.CSSProperties | null {
    if (aligns.length <= index) {
        return null;
    }
    const textAlign = aligns[index];
    if (textAlign === null) {
        return null;
    }
    return { textAlign };
}

export class ReactMarkdownRenderer {
    private table: TableState | null;
    private lastModifiedRef: React.RefObject<HTMLSpanElement> | null;
    private readonly footNotes: RenderTreeFootNoteDef[];
    private matchCount: number;
    private readonly fence: FenceRenderer;

    constructor(theme: Theme) {
        this.table = null;
        this.footNotes = [];
        this.lastModifiedRef = null;
        this.matchCount = 0;
        this.render = this.render.bind(this);
        this.fence = new FenceRenderer(theme);
    }

    async renderMarkdown(tree: RenderTreeElem[]): Promise<MarkdownReactTree> {
        log.debug('Rendering preview tree', tree);
        const blocks = await this.renderAll(tree);
        const footNotes = await this.renderFootnotes();
        const root = (
            <>
                {blocks}
                {footNotes}
            </>
        );
        return {
            root,
            lastModified: this.lastModifiedRef,
            matchCount: this.matchCount,
        };
    }

    private async renderFootnotes(): Promise<ReactNode> {
        if (this.footNotes.length === 0) {
            return null;
        }
        log.debug('Rendering footnotes', this.footNotes);

        const items = await Promise.all(
            this.footNotes.map(async (elem, idx) => {
                const children = await this.renderAll(elem.c);
                const backref = (
                    <a
                        href={`#user-content-fnref-${elem.id}`}
                        aria-label="Back to content"
                        key="backref"
                        style={FOOTNOTE_BACKREF_STYLE}
                    >
                        ↩
                    </a>
                );

                (lastElementOf(children)?.props?.children ?? children).push(backref);

                return (
                    <li key={idx} id={`user-content-fn-${elem.id}`}>
                        {children}
                    </li>
                );
            }),
        );

        return (
            <section className="footnotes">
                <h2 id="footnote-label">Footnotes</h2>
                <ol>{items}</ol>
            </section>
        );
    }

    private lastModified(key?: number): ReactNode {
        const ref = React.createRef<HTMLSpanElement>();
        this.lastModifiedRef = ref;
        return <span key={key} className="last-modified-marker" ref={ref} />;
    }

    private renderAll(elems: RenderTreeElem[]): Promise<ReactNode[]> {
        return Promise.all(elems.map((elem, idx) => this.render(elem, idx)));
    }

    private async render(elem: RenderTreeElem, key?: number): Promise<ReactNode> {
        if (typeof elem === 'string') {
            return elem;
        }

        switch (elem.t) {
            case 'p':
                return <p key={key}>{await this.renderAll(elem.c)}</p>;
            case 'h': {
                const tag = `h${elem.level}`;
                const props: JSX.IntrinsicElements['h1'] = { key };
                if (elem.id) {
                    props.id = elem.id; // TODO?: Clobber IDs
                }
                const children = await this.renderAll(elem.c);
                return React.createElement(tag, props, ...children);
            }
            case 'a':
                if (elem.auto) {
                    return (
                        <a key={key} href={elem.href}>
                            {await this.renderAll(elem.c)}
                        </a>
                    );
                } else {
                    // Note: material-ui's `Tooltip` component makes rendering this markdown preview 10x slower. Don't use it.
                    let title = elem.href;
                    if (elem.title && elem.title !== title) {
                        title = `"${elem.title}" ${title}`;
                    }
                    return (
                        <a key={key} title={title} href={elem.href}>
                            {await this.renderAll(elem.c)}
                        </a>
                    );
                }
            case 'img': {
                return <img key={key} src={elem.src} alt={rawText(elem)} title={elem.title} />;
            }
            case 'br':
                return <br key={key} />;
            case 'blockquote':
                return <blockquote key={key}>{await this.renderAll(elem.c)}</blockquote>;
            case 'em':
                return <em key={key}>{await this.renderAll(elem.c)}</em>;
            case 'strong':
                return <strong key={key}>{await this.renderAll(elem.c)}</strong>;
            case 'del':
                return <del key={key}>{await this.renderAll(elem.c)}</del>;
            case 'pre':
                return <pre key={key}>{await this.renderAll(elem.c)}</pre>;
            case 'code': {
                const rendered = await this.fence.render(elem, key);
                if (rendered === null) {
                    return <code key={key}>{await this.renderAll(elem.c)}</code>;
                }
                const [node, modified] = rendered;
                if (!modified) {
                    return node;
                }
                return (
                    <React.Fragment key={key}>
                        {this.lastModified()}
                        {node}
                    </React.Fragment>
                );
            }
            case 'ol':
                return (
                    <ol key={key} start={elem.start}>
                        {await this.renderAll(elem.c)}
                    </ol>
                );
            case 'ul':
                return <ul key={key}>{await this.renderAll(elem.c)}</ul>;
            case 'li':
                return <li key={key}>{await this.renderAll(elem.c)}</li>;
            case 'task-list':
                return (
                    <li key={key} className="task-list-item">
                        {await this.renderAll(elem.c)}
                    </li>
                );
            case 'emoji':
                return (
                    <span key={key} title={elem.name} role="img" aria-label={`${elem.name} emoji`}>
                        {await this.renderAll(elem.c)}
                    </span>
                );
            case 'table':
                this.table = {
                    aligns: elem.align,
                    index: 0,
                };
                return <table key={key}>{await this.renderAll(elem.c)}</table>;
            case 'thead':
                return <thead key={key}>{await this.renderAll(elem.c)}</thead>;
            case 'tbody':
                return <tbody key={key}>{await this.renderAll(elem.c)}</tbody>;
            case 'tr':
                if (this.table) {
                    this.table.index = 0;
                }
                return <tr key={key}>{await this.renderAll(elem.c)}</tr>;
            case 'th':
                if (this.table) {
                    const style = tableAlignStyle(this.table);
                    this.table.index++;
                    if (style !== null) {
                        return (
                            <th key={key} style={style}>
                                {await this.renderAll(elem.c)}
                            </th>
                        );
                    }
                }
                return <th key={key}>{await this.renderAll(elem.c)}</th>;
            case 'td':
                if (this.table) {
                    const style = tableAlignStyle(this.table);
                    this.table.index++;
                    if (style !== null) {
                        return (
                            <td key={key} style={style}>
                                {await this.renderAll(elem.c)}
                            </td>
                        );
                    }
                }
                return <td key={key}>{await this.renderAll(elem.c)}</td>;
            case 'checkbox': {
                return (
                    <input
                        key={key}
                        type="checkbox"
                        disabled
                        checked={elem.checked}
                        className="task-list-item-checkbox"
                    />
                );
            }
            case 'hr':
                return <hr key={key} />;
            case 'fn-ref':
                return (
                    <sup key={key}>
                        <a
                            href={`#user-content-fn-${elem.id}`}
                            id={`user-content-fnref-${elem.id}`}
                            aria-describedby="footnote-label"
                        >
                            {elem.id}
                        </a>
                    </sup>
                );
            case 'fn-def':
                this.footNotes.push(elem);
                return null; // Footnotes will be rendered at the bottom of page
            case 'math':
                if (elem.inline) {
                    return (
                        <span className="math-expr-inline" key={key}>
                            <Mathjax expr={elem.expr} />
                        </span>
                    );
                } else {
                    return (
                        <div className="math-expr-block" key={key}>
                            <Mathjax expr={elem.expr} />
                        </div>
                    );
                }
            case 'html': {
                // XXX: This <span> element is necessary because React cannot render inner HTML under fragment
                // https://github.com/reactjs/rfcs/pull/129
                // XXX: Relative paths don't work because they need to be adjusted with base directory path
                return <span key={key} dangerouslySetInnerHTML={{ __html: elem.raw }} />; // eslint-disable-line @typescript-eslint/naming-convention
            }
            case 'modified':
                return this.lastModified(key);
            case 'match':
                return (
                    <span key={key} className="search-text">
                        {await this.renderAll(elem.c)}
                    </span>
                );
            case 'match-current':
                return (
                    <span key={key} className="search-text-current">
                        {await this.renderAll(elem.c)}
                    </span>
                );
            case 'match-start':
                this.matchCount++;
                return (
                    <span key={key} className="search-text-start">
                        {await this.renderAll(elem.c)}
                    </span>
                );
            case 'match-current-start':
                this.matchCount++;
                return (
                    <span key={key} className="search-text-current-start">
                        {await this.renderAll(elem.c)}
                    </span>
                );
            default:
                log.error('Unknown render tree element:', JSON.stringify(elem));
                return null;
        }
    }
}
