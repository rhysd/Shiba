import hljs from 'highlight.js';
import tippy from 'tippy.js';
import { sanitize } from 'dompurify';
import mermaid from 'mermaid';
import * as log from './log';
import type { RenderTreeElem, ParseTreeTableAlign, ParseTreeFootNoteDef, ParseTreeCode } from './ipc';

function appearInViewport(elem: Element): boolean {
    const { top, left, bottom, right } = elem.getBoundingClientRect();
    const height = window.innerHeight ?? document.documentElement.clientHeight;
    const width = window.innerWidth ?? document.documentElement.clientWidth;
    const outside = bottom < 0 || height < top || right < 0 || width < left;
    return !outside;
}

export class PreviewContent {
    rootElem: HTMLElement;
    mermaidInit: boolean;

    constructor(root: HTMLElement) {
        this.rootElem = root;
        this.mermaidInit = false;
    }

    // Note: Render at requestAnimationFrame may be better for performance
    render(tree: RenderTreeElem[]): void {
        this.rootElem.textContent = '';
        const mermaid = new MermaidRenderer(this.mermaidInit);
        const renderer = new RenderTreeRenderer(mermaid);
        for (const elem of tree) {
            renderer.render(elem, this.rootElem);
        }
        renderer.end(this.rootElem);
        this.mermaidInit = mermaid.initialized;
        renderer.scrollToLastModified();
    }
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

function span(className: string): HTMLSpanElement {
    const s = document.createElement('span');
    s.className = className;
    return s;
}

interface TableState {
    aligns: ParseTreeTableAlign[];
    index: number;
}

class RenderTreeRenderer {
    table: TableState | null;
    footNotes: ParseTreeFootNoteDef[];
    lastModified: HTMLSpanElement | null;
    mermaid: MermaidRenderer;

    constructor(mermaid: MermaidRenderer) {
        this.table = null;
        this.footNotes = [];
        this.lastModified = null;
        this.mermaid = mermaid;
    }

    scrollToLastModified(): void {
        if (this.lastModified === null || appearInViewport(this.lastModified)) {
            return;
        }
        log.debug('Scrolling to last modified element:', this.lastModified);
        this.lastModified.scrollIntoView({
            behavior: 'smooth', // This does not work on WKWebView
            block: 'center',
            inline: 'center',
        });
    }

    tableAlign(): ParseTreeTableAlign {
        if (this.table === null) {
            return null;
        }
        const { aligns, index } = this.table;
        if (aligns.length >= index) {
            return null;
        }
        return aligns[index];
    }

    render(elem: RenderTreeElem, parent: HTMLElement): void {
        if (typeof elem === 'string') {
            parent.append(elem);
            return;
        }

        let node: HTMLElement;
        switch (elem.t) {
            case 'p':
                node = document.createElement('p');
                break;
            case 'h': {
                const h = document.createElement(`h${elem.level}`) as HTMLHeadingElement;
                if (elem.id) {
                    // TODO?: Clobber IDs
                    h.id = elem.id;
                }
                node = h;
                break;
            }
            case 'a': {
                const a = document.createElement('a');
                a.href = elem.href;
                let content = elem.href;
                let allowHTML = false;
                if (elem.title) {
                    a.title = elem.title;
                    content = `${elem.title}<br>${content}`;
                    allowHTML = true;
                }
                if (!elem.auto) {
                    tippy(a, { content, allowHTML });
                }
                node = a;
                break;
            }
            case 'img': {
                const i = document.createElement('img');
                i.src = elem.src;
                if (elem.title) {
                    i.title = elem.title;
                }
                i.alt = elem.c.map(rawText).join('');
                parent.appendChild(i);
                return;
            }
            case 'br':
                node = document.createElement('br');
                break;
            case 'blockquote':
                node = document.createElement('blockquote');
                break;
            case 'em':
                node = document.createElement('em');
                break;
            case 'strong':
                node = document.createElement('strong');
                break;
            case 'del':
                node = document.createElement('del');
                break;
            case 'pre':
                node = document.createElement('pre');
                break;
            case 'code': {
                const c = document.createElement('code');
                // When text search matches in content of code block, the highlight element is included in children.
                // highlight.js only allows highlighting text nodes.
                if (elem.lang && hljs.getLanguage(elem.lang) && elem.c.every(e => typeof e === 'string')) {
                    c.className = `language-${elem.lang}`;
                    for (const child of elem.c) {
                        this.render(child, c);
                    }
                    hljs.highlightElement(c);
                    parent.appendChild(c);
                    return;
                } else if (elem.lang === 'mermaid') {
                    if (this.mermaid.renderTo(parent, elem)) {
                        return;
                    }
                }
                node = c;
                break;
            }
            case 'ol': {
                const o = document.createElement('ol');
                if (typeof elem.start === 'number') {
                    o.start = elem.start;
                }
                node = o;
                break;
            }
            case 'ul':
                node = document.createElement('ul');
                break;
            case 'li':
                node = document.createElement('li');
                break;
            case 'emoji': {
                const s = document.createElement('span');
                s.title = elem.name;
                node = s;
                break;
            }
            case 'table': {
                this.table = {
                    aligns: elem.align,
                    index: 0,
                };
                node = document.createElement('table');
                break;
            }
            case 'thead':
                node = document.createElement('thead');
                break;
            case 'tbody':
                node = document.createElement('tbody');
                break;
            case 'tr':
                if (this.table) {
                    this.table.index = 0;
                }
                node = document.createElement('tr');
                break;
            case 'th':
                if (this.table) {
                    this.table.index++;
                }
                node = document.createElement('th');
                break;
            case 'td':
                if (this.table) {
                    this.table.index++;
                }
                node = document.createElement('td');
                break;
            case 'checkbox': {
                const i = document.createElement('input');
                i.type = 'checkbox';
                i.disabled = true;
                i.checked = elem.checked;
                node = i;
                break;
            }
            case 'hr':
                node = document.createElement('hr');
                break;
            case 'fn-ref': {
                const a = document.createElement('a');
                a.href = `#user-content-fn-${elem.id}`;
                a.id = `user-content-fnref-${elem.id}`;
                a.setAttribute('aria-describedby', 'footnote-label');
                a.textContent = `${elem.id}`;
                const sup = document.createElement('sup');
                sup.appendChild(a);
                node = sup;
                break;
            }
            case 'fn-def':
                this.footNotes.push(elem);
                return;
            case 'html': {
                const sanitized = sanitize(elem.raw, {
                    USE_PROFILES: { html: true },
                    RETURN_DOM_FRAGMENT: true,
                });
                parent.appendChild(sanitized);
                return;
            }
            case 'modified': {
                this.lastModified = span('last-modified-marker');
                node = this.lastModified;
                break;
            }
            case 'match': {
                node = span('search-text');
                break;
            }
            case 'match-current': {
                node = span('search-text-current');
                break;
            }
            case 'match-start': {
                node = span('search-text-start');
                break;
            }
            case 'match-current-start': {
                node = span('search-text-current-start');
                break;
            }
            default:
                log.error('Unknown render tree element:', JSON.stringify(elem));
                return;
        }

        if ('c' in elem) {
            for (const child of elem.c) {
                this.render(child, node);
            }
        }

        parent.appendChild(node);
    }

    end(parent: HTMLElement): void {
        if (this.footNotes.length === 0) {
            return;
        }
        const section = document.createElement('section');
        section.className = 'footnotes';

        {
            const heading = document.createElement('h2');
            heading.id = 'footnote-label';
            heading.className = 'sr-only';
            heading.textContent = 'Footnotes';
            section.appendChild(heading);
        }

        {
            const ol = document.createElement('ol');
            for (const elem of this.footNotes) {
                const li = document.createElement('li');
                li.id = `user-content-fn-${elem.id}`;

                for (const child of elem.c) {
                    this.render(child, li);
                }

                const a = document.createElement('a');
                a.textContent = '↩';
                a.href = `#user-content-fnref-${elem.id}`;
                a.className = 'data-footnote-backref';
                a.setAttribute('aria-label', 'Back to content');

                (li.lastChild ?? li).appendChild(a);
                ol.appendChild(li);
            }
            section.appendChild(ol);
        }

        parent.appendChild(section);
    }
}

class MermaidRenderer {
    initialized: boolean;
    id: number;

    constructor(init: boolean) {
        this.initialized = init;
        this.id = 0;
    }

    renderTo(parent: HTMLElement, elem: ParseTreeCode): boolean {
        let content = '';
        for (const child of elem.c) {
            if (typeof child === 'string') {
                content += child;
            } else {
                return false; // Reaches here when search highlight is included
            }
        }

        if (!this.initialized) {
            mermaid.initialize({ startOnLoad: false });
            this.initialized = true;
            log.debug('Initialized mermaid renderer for', elem);
        }

        const id = `graph-${this.id}`;
        this.id++;
        const svg = mermaid.render(id, content);
        parent.className = 'mermaid';
        parent.insertAdjacentHTML('beforeend', svg);

        return true;
    }
}
