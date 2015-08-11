/// <reference path="lib.d.ts" />
/// <reference path="../../typings/polymer/polymer.d.ts" />

const openExternal: (string) => void = require('shell').openExternal;
const path = require('path');

let element_env = null; // XXX
function openMarkdownLink(event: Event) {
    event.preventDefault();

    let path: string = (<HTMLAnchorElement>event.target).href;
    if (path.startsWith('file://')) {
        path = path.slice(7); // Omit 'file://'
    }

    if (element_env.openMarkdownDoc) {
        element_env.openMarkdownDoc(path);
    } else {
        console.log('openMarkdownDoc() is not defined!! Link ignored: ' + path);
    }
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
            value: []
        },

        openMarkdownDoc: Object
    },

    openLinkWithExternalBrowser: function(event) {
        event.preventDefault();
        if (event.target.href) {
            openExternal(event.target.href);
        } else if (event.target.src) {
            openExternal(event.target.src);
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

            if (link.href.startsWith('http')) {
                link.onclick = this.openLinkWithExternalBrowser;
                continue;
            }

            const ext = path.extname(link.href).slice(1); // Omit '.'
            if (this.exts.indexOf(ext) !== -1) {
                link.onclick = openMarkdownLink;
                continue;
            }

            // Note:
            // If the link is local link, it should not work
            link.onclick = event => event.preventDefault();
        }
    }
});
