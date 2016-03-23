/// <reference path="./emoji.ts" />
/// <reference path="lib.d.ts" />

import * as path from 'path';
import {unescape} from 'querystring';
import {shell} from 'electron';
import * as marked from 'marked';
import * as katex from 'katex';
import {highlight} from 'highlight.js';

const emoji_replacer = new Emoji.Replacer(path.dirname(__dirname) + '/images');

namespace MarkdownRenderer {
    marked.setOptions({
        highlight: function(code: string, lang: string): string {
            if (lang === undefined) {
                return code;
            }

            if (lang === 'mermaid') {
                return '<div class="mermaid">' + code + '</div>';
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
    });

    const REGEX_CHECKED_LISTITEM = /^\[x]\s+/;
    const REGEX_UNCHECKED_LISTITEM = /^\[ ]\s+/;
    const renderer = new marked.Renderer();

    renderer.listitem = function(text) {
        let matched = text.match(REGEX_CHECKED_LISTITEM);
        if (matched && matched[0]) {
            return '<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" checked="checked" disabled="disabled">'
                + text.slice(matched[0].length) + '</li>\n';
        }

        matched = text.match(REGEX_UNCHECKED_LISTITEM);
        if (matched && matched[0]) {
            return '<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" disabled="disabled">'
                + text.slice(matched[0].length) + '</li>\n';
        }

        return marked.Renderer.prototype.listitem.call(this, text);
    };

    renderer.text = function(text) {
        return emoji_replacer.replaceWithImages(text);
    };

    export function render(markdown: string): string {
        return marked(markdown, {renderer});
    }
}

interface LinkOpenerType {
    openMarkdownDoc(path: string, modifier: boolean): void;
}

let element_env: LinkOpenerType = null; // XXX
function openMarkdownLink(event: MouseEvent) {
    event.preventDefault();

    const anchor = event.target as HTMLAnchorElement;

    let path = unescape(anchor.href);
    if (path.startsWith('file://')) {
        path = path.slice(7); // Omit 'file://'
    }

    if (element_env.openMarkdownDoc) {
        element_env.openMarkdownDoc(path, event.ctrlKey || event.metaKey);
    } else {
        console.log('openMarkdownDoc() is not defined!! Link ignored: ' + path);
    }
}

function openHashLink(event: MouseEvent) {
    event.preventDefault();

    let hash_name: string = (<HTMLAnchorElement>event.target).href.split('#')[1];
    location.hash = hash_name;
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
            value: function(){ return [] as string[]; },
        },

        openMarkdownDoc: Object,

        fontSize: String,
    },

    openLinkWithExternalBrowser: function(event) {
        event.preventDefault();
        const e = event.target as HTMLElement & {src?: string; href?: string};
        shell.openExternal(e.href ? e.href : e.src);
    },

    ready: function() {
        element_env = this; // XXX
    },

    attached: function() {
        const body = document.querySelector('.markdown-body') as HTMLDivElement;
        body.style.fontSize = this.fontSize;
    },

    _documentUpdated: function(updated_doc) {
        console.log('foo!');
        const body = document.querySelector('.markdown-body') as HTMLDivElement;
        body.innerHTML = MarkdownRenderer.render(updated_doc);

        // Prevent external links from opening in page
        const links = body.querySelectorAll('a');
        for (let i = 0; i < links.length; ++i) {
            const link = links.item(i) as HTMLAnchorElement;
            if (!link.href) {
                continue;
            }

            // Note: External link
            if (link.href.startsWith('http')) {
                link.onclick = this.openLinkWithExternalBrowser;
                continue;
            }

            // Note: Link to local markdown document
            const ext_idx = link.href.lastIndexOf('.');
            if (ext_idx !== -1) {
                const ext = link.href.slice(ext_idx + 1);
                if (this.exts.indexOf(ext) !== -1) {
                    link.onclick = openMarkdownLink;
                    continue;
                }
            }

            // Note: Inner link (<base> tag appends prefix to the hash name)
            if (link.href.indexOf('#') !== -1) {
                link.onclick = openHashLink;
                continue;
            }

            // Note:
            // If the link is local link, it should not work
            link.onclick = event => event.preventDefault();
        }

        if (document.querySelector('.lang-mermaid') !== null) {
            mermaid.init();
        }
    },
} as MarkdownPreviewComponent);
