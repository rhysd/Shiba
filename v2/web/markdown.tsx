import React from 'react';
import hljs from 'highlight.js';
import { sanitize } from 'dompurify';
import type { RenderTreeElem, RenderTreeFootNoteDef, RenderTreeTableAlign } from './ipc';
import * as log from './log';
import { CodeBlock } from './components/CodeBlock';
import { Mathjax } from './components/Mathjax';
import { Mermaid } from './components/Mermaid';
import { Link } from './components/Link';

const FOOTNOTE_BACKREF_STYLE: React.CSSProperties = {
    fontFamily: 'monospace',
    fontSize: '1.25em',
    margin: '0 0.25em',
};

export interface MarkdownReactTree {
    root: React.ReactNode;
    lastModified: React.RefObject<HTMLSpanElement> | null;
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

function isReactElement(node: React.ReactNode): node is React.ReactElement {
    return node !== null && typeof node === 'object' && '$$typeof' in node;
}

function lastElementOf(nodes: React.ReactNode[]): React.ReactElement | null {
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
    table: TableState | null;
    lastModifiedRef: React.RefObject<HTMLSpanElement> | null;
    readonly footNotes: RenderTreeFootNoteDef[];

    constructor() {
        this.table = null;
        this.footNotes = [];
        this.lastModifiedRef = null;
        this.render = this.render.bind(this);
    }

    renderMarkdown(tree: RenderTreeElem[]): MarkdownReactTree {
        log.debug('Rendering preview tree', tree);
        const blocks = tree.map(this.render);
        const root = (
            <>
                {blocks}
                {this.renderFootnotes()}
            </>
        );
        return { root, lastModified: this.lastModifiedRef };
    }

    renderFootnotes(): React.ReactNode {
        if (this.footNotes.length === 0) {
            return undefined;
        }
        log.debug('Rendering footnotes', this.footNotes);

        const items = this.footNotes.map((elem, idx) => {
            const children = elem.c.map(this.render);
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

            const last = lastElementOf(children);
            if (last !== null) {
                last.props ??= {};
                last.props.children ??= [];
                last.props.children.push(backref);
            } else {
                children.push(backref);
            }

            return (
                <li key={idx} id={`user-content-fn-${elem.id}`}>
                    {children}
                </li>
            );
        });
        return (
            <section className="footnotes">
                <h2 id="footnote-label">Footnotes</h2>
                <ol>{items}</ol>
            </section>
        );
    }

    lastModified(key?: number): React.ReactNode {
        const ref = React.createRef<HTMLSpanElement>();
        this.lastModifiedRef = ref;
        return <span key={key} className="last-modified-marker" ref={ref} />;
    }

    maybeModified(elem: React.ReactElement, modified: boolean, key: number | undefined): React.ReactNode {
        if (!modified) {
            return elem;
        }
        return (
            <React.Fragment key={key}>
                {this.lastModified()}
                {elem}
            </React.Fragment>
        );
    }

    render(elem: RenderTreeElem, key?: number): React.ReactNode {
        if (typeof elem === 'string') {
            return elem;
        }

        switch (elem.t) {
            case 'p':
                return <p key={key}>{elem.c.map(this.render)}</p>;
            case 'h': {
                const tag = `h${elem.level}`;
                const props: JSX.IntrinsicElements['h1'] = { key };
                if (elem.id) {
                    props.id = elem.id; // TODO?: Clobber IDs
                }
                return React.createElement(tag, props, ...elem.c.map(this.render));
            }
            case 'a':
                if (elem.auto) {
                    return (
                        <a key={key} href={elem.href}>
                            {elem.c.map(this.render)}
                        </a>
                    );
                } else {
                    return (
                        <Link key={key} title={elem.title} href={elem.href}>
                            {elem.c.map(this.render)}
                        </Link>
                    );
                }
            case 'img': {
                return <img key={key} src={elem.src} alt={rawText(elem)} title={elem.title} />;
            }
            case 'br':
                return <br key={key} />;
            case 'blockquote':
                return <blockquote key={key}>{elem.c.map(this.render)}</blockquote>;
            case 'em':
                return <em key={key}>{elem.c.map(this.render)}</em>;
            case 'strong':
                return <strong key={key}>{elem.c.map(this.render)}</strong>;
            case 'del':
                return <del key={key}>{elem.c.map(this.render)}</del>;
            case 'pre':
                return <pre key={key}>{elem.c.map(this.render)}</pre>;
            case 'code':
                if (elem.lang) {
                    if (hljs.getLanguage(elem.lang)) {
                        const text = childrenText(elem.c);
                        if (text !== null) {
                            const [content, modified] = text;
                            return this.maybeModified(
                                <CodeBlock key={key} lang={elem.lang} code={content} />,
                                modified,
                                key,
                            );
                        }
                    } else if (elem.lang === 'mermaid') {
                        const text = childrenText(elem.c);
                        if (text !== null) {
                            const [content, modified] = text;
                            return this.maybeModified(<Mermaid key={key} content={content} />, modified, key);
                        }
                    } else if (elem.lang === 'math') {
                        const text = childrenText(elem.c);
                        if (text !== null) {
                            const [content, modified] = text;
                            return this.maybeModified(
                                <Mathjax key={key} expr={content} className="code-fence-math" />,
                                modified,
                                key,
                            );
                        }
                    }
                }
                return <code key={key}>{elem.c.map(this.render)}</code>;
            case 'ol': {
                return (
                    <ol key={key} start={elem.start}>
                        {elem.c.map(this.render)}
                    </ol>
                );
            }
            case 'ul':
                return <ul key={key}>{elem.c.map(this.render)}</ul>;
            case 'li':
                return <li key={key}>{elem.c.map(this.render)}</li>;
            case 'emoji': {
                return (
                    <span key={key} title={elem.name}>
                        {elem.c.map(this.render)}
                    </span>
                );
            }
            case 'table': {
                this.table = {
                    aligns: elem.align,
                    index: 0,
                };
                return <table key={key}>{elem.c.map(this.render)}</table>;
            }
            case 'thead':
                return <thead key={key}>{elem.c.map(this.render)}</thead>;
            case 'tbody':
                return <tbody key={key}>{elem.c.map(this.render)}</tbody>;
            case 'tr':
                if (this.table) {
                    this.table.index = 0;
                }
                return <tr key={key}>{elem.c.map(this.render)}</tr>;
            case 'th':
                if (this.table) {
                    const style = tableAlignStyle(this.table);
                    this.table.index++;
                    if (style !== null) {
                        return (
                            <th key={key} style={style}>
                                {elem.c.map(this.render)}
                            </th>
                        );
                    }
                }
                return <th key={key}>{elem.c.map(this.render)}</th>;
            case 'td':
                if (this.table) {
                    const style = tableAlignStyle(this.table);
                    this.table.index++;
                    if (style !== null) {
                        return (
                            <td key={key} style={style}>
                                {elem.c.map(this.render)}
                            </td>
                        );
                    }
                }
                return <td key={key}>{elem.c.map(this.render)}</td>;
            case 'checkbox': {
                return <input key={key} type="checkbox" disabled checked={elem.checked} />;
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
                return <React.Fragment key={key}></React.Fragment>; // Footnotes will be rendered at the bottom of page
            case 'math':
                return (
                    <Mathjax
                        key={key}
                        expr={elem.expr}
                        className={elem.inline ? 'math-expr-inline' : 'math-expr-block'}
                    />
                );
            case 'html': {
                const sanitized = sanitize(elem.raw, { USE_PROFILES: { html: true } });
                // XXX: This <span> element is necessary because React cannot render inner HTML under fragment
                // https://github.com/reactjs/rfcs/pull/129
                return <span key={key} dangerouslySetInnerHTML={{ __html: sanitized }} />; // eslint-disable-line @typescript-eslint/naming-convention
            }
            case 'modified':
                return this.lastModified(key);
            case 'match':
                return (
                    <span key={key} className="search-text">
                        {elem.c.map(this.render)}
                    </span>
                );
            case 'match-current':
                return (
                    <span key={key} className="search-text-current">
                        {elem.c.map(this.render)}
                    </span>
                );
            case 'match-start':
                return (
                    <span key={key} className="search-text-start">
                        {elem.c.map(this.render)}
                    </span>
                );
            case 'match-current-start':
                return (
                    <span key={key} className="search-text-current-start">
                        {elem.c.map(this.render)}
                    </span>
                );
            default:
                log.error('Unknown render tree element:', JSON.stringify(elem));
                return <React.Fragment key={key}></React.Fragment>;
        }
    }
}
