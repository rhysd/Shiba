import * as React from 'react';
import type { ReactElement } from 'react';
import hljs from 'highlight.js';
import mermaid from 'mermaid';
import { mathjax } from 'mathjax-full/js/mathjax';
import type { MathDocument } from 'mathjax-full/js/core/MathDocument';
import { TeX } from 'mathjax-full/js/input/tex';
import { SVG } from 'mathjax-full/js/output/svg';
import { AllPackages } from 'mathjax-full/js/input/tex/AllPackages';
import { liteAdaptor, type LiteAdaptor } from 'mathjax-full/js/adaptors/liteAdaptor';
import { RegisterHTMLHandler } from 'mathjax-full/js/handlers/html';
import type { LiteElement } from 'mathjax-full/js/adaptors/lite/Element';
import type { LiteText } from 'mathjax-full/js/adaptors/lite/Text';
import type { LiteDocument } from 'mathjax-full/js/adaptors/lite/Document';
import type { RenderTreeElem, RenderTreeCodeFence, WindowTheme } from './ipc';
import { Mermaid } from './components/Mermaid';
import * as log from './log';

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

export class MermaidRenderer {
    private theme: 'default' | 'dark' = 'default';
    private initialized = false;
    private id = 0;

    setTheme(theme: WindowTheme): void {
        const next = theme === 'Light' ? 'default' : 'dark';
        if (this.theme !== next && this.initialized) {
            this.initialized = false;
            log.debug('Mermaid will be initialized again since the window theme was changed', this.theme, next, theme);
        }
        this.theme = next;
    }

    resetId(): void {
        this.id = 0;
    }

    private initMermaid(): void {
        if (this.initialized) {
            return;
        }
        mermaid.initialize({ startOnLoad: false, theme: this.theme });
        log.debug('Initialized mermaid renderer', this.theme);
        this.initialized = true;
    }

    async render(content: string, key: number | undefined): Promise<ReactElement> {
        this.initMermaid();
        const id = this.id++;
        const { svg, bindFunctions } = await mermaid.render(`graph-${id}`, content);
        return <Mermaid svg={svg} bindFn={bindFunctions} key={key} />;
    }
}

type MathJaxDocument = MathDocument<LiteElement, LiteText, LiteDocument>;
type MathJaxState = [MathJaxDocument, LiteAdaptor];
type MathClassName = 'math-expr-block' | 'math-expr-inline' | 'code-fence-math';

export class MathJaxRenderer {
    private state: MathJaxState | null = null;

    private initMathJax(): MathJaxState {
        if (this.state !== null) {
            return this.state;
        }

        const adaptor = liteAdaptor();
        RegisterHTMLHandler(adaptor);
        const document = mathjax.document('', {
            InputJax: new TeX({ packages: AllPackages }),
            OutputJax: new SVG({ fontCache: 'local' }),
        });
        this.state = [document, adaptor];
        return this.state;
    }

    render(expr: string, className: MathClassName, key: number | undefined): ReactElement {
        const [document, adaptor] = this.initMathJax();
        const node = document.convert(expr) as LiteElement;
        const html = adaptor.innerHTML(node);
        return <span className={className} dangerouslySetInnerHTML={{ __html: html }} key={key} />; // eslint-disable-line @typescript-eslint/naming-convention
    }
}

export class FenceRenderer {
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
            const rendered = this.mathjax.render(content, 'code-fence-math', key);
            return [rendered, modified];
        }

        return null;
    }
}
