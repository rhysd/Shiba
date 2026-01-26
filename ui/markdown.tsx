import * as React from 'react';
import type { ReactNode, ReactElement } from 'react';
import hljs from 'highlight.js';
import mermaid from 'mermaid';
import { mathjax } from '@mathjax/src/cjs/mathjax.js';
import type { MathDocument } from '@mathjax/src/cjs/core/MathDocument.js';
import { TeX } from '@mathjax/src/cjs/input/tex.js';
import { SVG } from '@mathjax/src/cjs/output/svg.js';
import { liteAdaptor, type LiteAdaptor } from '@mathjax/src/cjs/adaptors/liteAdaptor.js';
import { RegisterHTMLHandler } from '@mathjax/src/cjs/handlers/html.js';
import type { LiteElement } from '@mathjax/src/cjs/adaptors/lite/Element.js';
import type { LiteText } from '@mathjax/src/cjs/adaptors/lite/Text.js';
import type { LiteDocument } from '@mathjax/src/cjs/adaptors/lite/Document.js';
import { MathJaxTexFont } from '@mathjax/mathjax-tex-font/cjs/svg.js';
import { InfoIcon, LightBulbIcon, AlertIcon, ReportIcon, StopIcon } from '@primer/octicons-react';
import type {
    RenderTreeElem,
    RenderTreeFootNoteDef,
    RenderTreeTableAlign,
    RenderTreeCodeFence,
    AlertKind,
} from './ipc';
import { colorScheme } from './css';
import * as log from './log';
import { Mermaid } from './components/Mermaid';

class MermaidRenderer {
    private initialized = false;
    private id = 0;

    constructor() {
        colorScheme.addListener(() => {
            this.initialized = false;
        });
    }

    resetId(): void {
        this.id = 0;
    }

    private initMermaid(): void {
        if (this.initialized) {
            return;
        }
        const theme = colorScheme.isDark ? 'dark' : 'default';
        mermaid.initialize({ startOnLoad: false, theme });
        log.debug('Initialized mermaid renderer', theme);
        this.initialized = true;
    }

    async render(content: string, key: number | undefined): Promise<ReactElement> {
        this.initMermaid();

        try {
            // This throws an exception when parse error happens. The boolean return value is useless
            // when `suppressErrors` is not set to `parseOptions` argument of `mermaid.parse`.
            await mermaid.parse(content);
        } catch (err) {
            return <span key={key}>Diagram rendering error: {String(err)}</span>;
        }

        const id = this.id++;
        const { svg, bindFunctions } = await mermaid.render(`graph-${id}`, content);
        return <Mermaid svg={svg} bindFn={bindFunctions} key={key} />;
    }
}

type MathJaxDocument = MathDocument<LiteElement, LiteText, LiteDocument>;
type MathJaxState = [MathJaxDocument, LiteAdaptor];
type MathClassName = 'math-expr-block' | 'math-expr-inline' | 'code-fence-math';

class MathJaxRenderer {
    private state: MathJaxState | null = null;

    private async initMathJax(): Promise<MathJaxState> {
        if (this.state !== null) {
            return this.state;
        }

        await import('@mathjax/src/cjs/input/tex/ams/AmsConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/amscd/AmsCdConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/boldsymbol/BoldsymbolConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/braket/BraketConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/bussproofs/BussproofsConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/cancel/CancelConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/cases/CasesConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/centernot/CenternotConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/color/ColorConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/empheq/EmpheqConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/enclose/EncloseConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/extpfeil/ExtpfeilConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/gensymb/GensymbConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/mathtools/MathtoolsConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/mhchem/MhchemConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/noundefined/NoUndefinedConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/upgreek/UpgreekConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/unicode/UnicodeConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/verb/VerbConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/configmacros/ConfigMacrosConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/tagformat/TagFormatConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/textcomp/TextcompConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/textmacros/TextMacrosConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/physics/PhysicsConfiguration.js');
        await import('@mathjax/src/cjs/input/tex/newcommand/NewcommandConfiguration.js');

        const packages = [
            // The list of TeX packages actually used on github.com. To retrieve this list
            //
            // 1. Open some Markdown text area
            // 2. Input some math expressions and show the preview
            // 3. Open DevTools and see `JSON.stringify(window.MathJax.config.tex.packages)`
            //
            // Note that github.com still seems to utilize MathJax v3.
            'base',
            'ams',
            'amscd',
            'boldsymbol',
            'braket',
            'bussproofs',
            'cancel',
            'cases',
            'centernot',
            'color',
            'empheq',
            'enclose',
            'extpfeil',
            'gensymb',
            'mathtools',
            'mhchem',
            'noundefined',
            'upgreek',
            'unicode',
            'verb',
            'configmacros',
            'tagformat',
            'textcomp',
            'textmacros',
            // Additional popular packages
            'physics',
            'newcommand',
        ];

        const adaptor = liteAdaptor();
        RegisterHTMLHandler(adaptor);
        const document = mathjax.document('', {
            InputJax: new TeX({ packages }),
            OutputJax: new SVG({ fontCache: 'local', fontData: MathJaxTexFont }),
        });
        this.state = [document, adaptor];
        return this.state;
    }

    async render(expr: string, className: MathClassName, key: number | undefined): Promise<ReactElement> {
        const [document, adaptor] = await this.initMathJax();
        const node = await document.convertPromise(expr);
        const html = adaptor.innerHTML(node as LiteElement);
        return <span className={className} dangerouslySetInnerHTML={{ __html: html }} key={key} />; // eslint-disable-line @typescript-eslint/naming-convention
    }
}

class FenceRenderer {
    private readonly mermaid: MermaidRenderer;
    private readonly mathjax: MathJaxRenderer;

    constructor(mermaid: MermaidRenderer, mathjax: MathJaxRenderer) {
        this.mermaid = mermaid;
        this.mathjax = mathjax;
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
            const rendered = await this.mermaid.render(content, key);
            return [rendered, modified];
        }

        if (elem.lang === 'math') {
            const text = childrenText(elem.c);
            if (text === null) {
                return null;
            }
            const [content, modified] = text;
            const rendered = await this.mathjax.render(content, 'code-fence-math', key);
            return [rendered, modified];
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
    lastModified: React.RefObject<HTMLSpanElement | null> | null;
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

interface ReactElementProps {
    children?: ReactNode[];
    [key: string]: unknown;
}

function isReactElement(node: ReactNode): node is ReactElement<ReactElementProps> {
    return node !== null && typeof node === 'object' && Object.prototype.hasOwnProperty.call(node, '$$typeof');
}

function lastElementOf(nodes: ReactNode[]): ReactElement<ReactElementProps> | null {
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

function alertIcon(kind: AlertKind): ReactElement | null {
    switch (kind) {
        case 'note':
            return <InfoIcon className="octicon octicon-info mr-2" />;
        case 'tip':
            return <LightBulbIcon className="octicon octicon-light-bulb mr-2" />;
        case 'warning':
            return <AlertIcon className="octicon octicon-alert mr-2" />;
        case 'important':
            return <ReportIcon className="octicon octicon-report mr-2" />;
        case 'caution':
            return <StopIcon className="octicon octicon-stop mr-2" />;
        default:
            return null;
    }
}

class RenderTreeToReact {
    private table: TableState | null;
    private lastModifiedRef: React.RefObject<HTMLSpanElement | null> | null;
    private readonly footNotes: RenderTreeFootNoteDef[];
    private matchCount: number;
    private readonly fence: FenceRenderer;
    private readonly mathjax: MathJaxRenderer;

    constructor(mermaid: MermaidRenderer, mathjax: MathJaxRenderer) {
        this.table = null;
        this.footNotes = [];
        this.lastModifiedRef = null;
        this.matchCount = 0;
        this.render = this.render.bind(this);
        this.fence = new FenceRenderer(mermaid, mathjax);
        this.mathjax = mathjax;
    }

    async run(tree: RenderTreeElem[]): Promise<MarkdownReactTree> {
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
                        aria-label={`Back to reference ${elem.id}`}
                        key="backref"
                        style={FOOTNOTE_BACKREF_STYLE}
                    >
                        ↩
                    </a>
                );

                (lastElementOf(children)?.props.children ?? children).push(backref);

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
                const props: React.JSX.IntrinsicElements['h1'] = { key };
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
                            [{elem.id}]
                        </a>
                    </sup>
                );
            case 'fn-def':
                this.footNotes.push(elem);
                return null; // Footnotes will be rendered at the bottom of page
            case 'math': {
                const className = elem.inline ? 'math-expr-inline' : 'math-expr-block';
                return this.mathjax.render(elem.expr, className, key);
            }
            case 'alert': {
                const className = `markdown-alert markdown-alert-${elem.kind}`;
                const title = elem.kind.charAt(0).toUpperCase() + elem.kind.slice(1);
                return (
                    <div className={className} key={key}>
                        <p className="markdown-alert-title">
                            {alertIcon(elem.kind)}
                            {title}
                        </p>
                        {await this.renderAll(elem.c)}
                    </div>
                );
            }
            case 'html':
                // When an HTML sanitizer dropped an entire input, the result can be empty.
                if (elem.raw.length === 0) {
                    return null;
                }
                // XXX: This <span> element is necessary because React cannot render inner HTML under fragment
                // https://github.com/reactjs/rfcs/pull/129
                // XXX: Relative paths don't work because they need to be adjusted with base directory path
                return <span key={key} dangerouslySetInnerHTML={{ __html: elem.raw }} />; // eslint-disable-line @typescript-eslint/naming-convention
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

export class ReactMarkdownRenderer {
    private readonly mermaid = new MermaidRenderer();
    private readonly mathjax = new MathJaxRenderer();

    render(tree: RenderTreeElem[]): Promise<MarkdownReactTree> {
        this.mermaid.resetId();
        const renderer = new RenderTreeToReact(this.mermaid, this.mathjax);
        return renderer.run(tree);
    }
}
