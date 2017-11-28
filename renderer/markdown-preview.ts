/// <reference path="./emoji.ts" />
/// <reference path="lib.d.ts" />

import {dirname} from 'path';
import {unescape} from 'querystring';
import {shell, remote} from 'electron';
import * as marked from 'marked';
import * as katex from 'katex';
import {highlight} from 'highlight.js';
import * as he from 'he';

let element_env: MarkdownPreview = null; // XXX
const emoji_replacer = new Emoji.Replacer(dirname(__dirname) + '/images');
let mermaid: any = undefined;

marked.setOptions({
    langPrefix: 'hljs ',

    highlight(code: string, lang: string): string {
        if (lang === undefined) {
            return code;
        }

        if (lang === 'mermaid') {
            if (mermaid === undefined) {
                mermaid = require('mermaid');
                mermaid.init(undefined, 'div.mermaid');
            }
            return '<div class="mermaid">' + he.encode(code) + '</div>';
        }

        if (lang === 'katex') {
            return '<div class="katex">' + katex.renderToString(code, {displayMode: true}) + '</div>';
        }

        try {
            return highlight(lang, code).value;
        } catch (e) {
            console.log('Error on highlight: ' + e.message);
            return code;
        }
    },

    // @ts-ignore: emoji is a dedicated method added by my fork
    emoji(name: string) {
        return emoji_replacer.replaceOne(name);
    },

    sanitize: 1,
});

const REGEX_CHECKED_LISTITEM = /^\[x]\s+/;
const REGEX_UNCHECKED_LISTITEM = /^\[ ]\s+/;

class MarkdownRenderer {
    public outline: Heading[];
    private renderer: marked.Renderer;
    private link_id: number;
    private tooltips: string;

    constructor(public markdown_exts: string[]) {
        this.renderer = new marked.Renderer();

        // TODO:
        // 'this' is set to renderer methods automatically so we need to preserve
        // this scope's 'this' as 'self'.
        /* tslint:disable:no-this-assignment */
        const self = this;
        /* tslint:enable:no-this-assignment */

        this.renderer.listitem = function(text) {
            let matched = text.match(REGEX_CHECKED_LISTITEM);
            if (matched && matched[0]) {
                return `
                  <li class="task-list-item">
                    <input type="checkbox" class="task-list-item-checkbox" checked="checked" disabled="disabled">
                    ${text.slice(matched[0].length)}
                  </li>
                `;
            }

            matched = text.match(REGEX_UNCHECKED_LISTITEM);
            if (matched && matched[0]) {
                return `
                  <li class="task-list-item">
                    <input type="checkbox" class="task-list-item-checkbox" disabled="disabled">
                    ${text.slice(matched[0].length)}
                  </li>
                `;
            }

            return marked.Renderer.prototype.listitem.call(this, text);
        };

        const re_ext = new RegExp(`\\.(:?${this.markdown_exts.join('|')})(:?$|#)`);
        this.renderer.link = function(href, title, text) {
            if (!href) {
                return marked.Renderer.prototype.link.call(this, href, title, text);
            }

            if (this.options.sanitize) {
                try {
                    const prot = decodeURIComponent(unescape(href))
                        .replace(/[^\w:]/g, '')
                        .toLowerCase();
                    if (prot.startsWith('javascript:') || prot.startsWith('vbscript:')) {
                        return '';
                    }
                } catch (e) {
                    return '';
                }
            }

            let onclick = 'cancelClick(event)';
            if (href.startsWith('http://') || href.startsWith('https://')) {
                onclick = `openLinkWithExternalBrowser(event, this.id)`;
            } else if (re_ext.test(href)) {
                onclick = 'openMarkdownLink(event)';
            } else if (href.indexOf('#') !== -1) {
                onclick = 'openHashLink(event)';
            }

            self.link_id += 1;
            const id = `md-link-${self.link_id}`;
            self.tooltips += `<paper-tooltip for="${id}" offset="0">${he.encode(href)}</paper-tooltip>`;

            return title !== null ?
                `<a id="${id}" href="${href}" onclick="${onclick}" title=${title}>${text}</a>` :
                `<a id="${id}" href="${href}" onclick="${onclick}">${text}</a>`;
        };

        this.renderer.heading = function(text, level, raw) {
            const hash = this.options.headerPrefix + raw.toLowerCase().replace(/[^\w]+/g, '-');
            self.outline.push({
                title: raw,
                hash,
                level,
                html: text,
            });
            return marked.Renderer.prototype.heading.call(this, text, level, raw);
        };
    }

    render(markdown: string): string {
        this.link_id = 0;
        this.tooltips = '';
        this.outline = [];
        return marked(markdown, {renderer: this.renderer}) + this.tooltips;
    }
}

/* tslint:disable:no-unused-variable */
function openMarkdownLink(event: MouseEvent) {
/* tslint:enable:no-unused-variable */
    event.preventDefault();

    let target = event.target as HTMLAnchorElement;
    while (target !== null) {
        if (target.href) {
            break;
        }
        target = target.parentElement as HTMLAnchorElement;
    }

    if (target === null) {
        console.log('openMarkdownLink(): No target <a> was found', event);
        return;
    }

    let path = unescape(target.href);
    if (path.startsWith('file://')) {
        path = path.slice(7); // Omit 'file://'
    }

    const hash_idx = path.indexOf('#');
    if (hash_idx !== -1) {
        path = path.slice(0, hash_idx);
    }

    if (process.platform === 'win32' && path[0] === '/') {
        // Chromium convert relative path of 'href' into absolute path.
        // But on Windows 'foo/bar' is converted into 'file:///C:/foo/bar'.
        // C:/foo/bar is correct. So strip first '/' here (#37).
        path = path.slice(1);
    }

    if (element_env.openMarkdownDoc) {
        element_env.openMarkdownDoc(path, event.ctrlKey || event.metaKey);
    } else {
        console.log('openMarkdownDoc() is not defined!! Link ignored: ' + path);
    }
}

/* tslint:disable:no-unused-variable */
function openHashLink(event: MouseEvent) {
/* tslint:enable:no-unused-variable */
    event.preventDefault();
    const target = event.target as HTMLAnchorElement;
    const hash_name: string = target.href.split('#')[1];
    location.hash = hash_name;
}

/* tslint:disable:no-unused-variable */
function openLinkWithExternalBrowser(event: MouseEvent, id: string) {
/* tslint:enable:no-unused-variable */
    event.preventDefault();
    const e = document.getElementById(id) as HTMLAnchorElement;
    shell.openExternal(e.href);
}

/* tslint:disable:no-unused-variable */
function cancelClick(event: MouseEvent) {
/* tslint:enable:no-unused-variable */
    event.preventDefault();
}

Polymer({
    is: 'markdown-preview',

    properties: {
        document: {
            type: String,
            observer: '_documentUpdated',
        },

        exts: {
            type: Array,
            value() { return [] as string[]; },
        },

        currentOutline: {
            type: Array,
            value() { return [] as Heading[]; },
        },

        isGithubStyle: {
            type: Boolean,
            value: true,
        },

        openMarkdownDoc: Object,
        fontSize: String,
        onDocumentUpdated: Object,
    },

    ready() {
        element_env = this; // XXX
    },

    attached() {
        this.renderer = new MarkdownRenderer(this.exts);
        const body = document.getElementById('shiba-markdown-component') as HTMLDivElement;
        if (this.fontSize) {
            body.style.fontSize = this.fontSize;
        }
        if (!this.isGithubStyle) {
            body.classList.remove('markdown-body');
        }
    },

    _documentUpdated(updated_doc) {
        const body = document.getElementById('shiba-markdown-component') as HTMLDivElement;
        body.innerHTML = this.renderer.render(updated_doc);
        this.currentOutline = this.renderer.outline;
        const mermaid_blocks = document.querySelectorAll('div.mermaid');
        if (mermaid_blocks.length > 0 && typeof(mermaid) !== 'undefined') {
            mermaid.init(undefined, mermaid_blocks);
        }
        if (this.onDocumentUpdated) {
            this.onDocumentUpdated();
        }
    },

    scrollToHeading(scroller: Scroller, h: Heading) {
        const elem = document.getElementById(h.hash);
        if (elem) {
            scroller.scrollTop = elem.offsetTop;
        }
    },
} as MarkdownPreviewComponent);
