/// <reference path="lib.d.ts" />

const openExternal = require('shell').openExternal as (url: string) => void;
const unescape = require('querystring').unescape as (str: string) => string;

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

interface MarkdownPreviewComponent extends polymer.Base {
    openLinkWithExternalBrowser(event: Event): void;
    _contentUpdated(new_content: string): void;
}

Polymer({
    is: 'markdown-preview',

    properties: {
        content: {
            type: String,
            observer: '_contentUpdated'
        },

        exts: {
            type: Array,
            value: function(){ return [] as string[]; },
        },

        openMarkdownDoc: Object
    },

    openLinkWithExternalBrowser: function(event) {
        event.preventDefault();
        let e = event.target as HTMLElement & {src?: string; href?: string};
        if (e.href) {
            openExternal(e.href);
        } else if (e.src) {
            openExternal(e.src);
        }
    },

    ready: function() {
        element_env = this; // XXX
    },

    _contentUpdated: function(new_content) {
        let body = <HTMLDivElement>document.querySelector('.markdown-body');
        body.innerHTML = new_content;

        // Prevent external links from opening in page
        const links = document.querySelectorAll('a');
        for (let i = 0; i < links.length; ++i) {
            let link = <HTMLAnchorElement>links.item(i);
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

        if (document.querySelector('.lang-mermaid') !== undefined) {
            mermaid.init();
        }
    }
} as MarkdownPreviewComponent);
