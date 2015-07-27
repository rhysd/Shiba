/// <reference path="keyreceiver.ts" />
/// <reference path="lib.d.ts" />

var remote = require('remote');

function onPathButtonPushed(): void {
    document.getElementById('path-change').open();
}

function makeTitle(path: string): string {
    if (path === '') {
        return 'Shiba';
    } else {
        return 'Shiba (' + path + ')';
    }
}

function getScroller() {
    const selected: string = document.getElementById('main-drawer').selected;
    if (selected === null) {
        return null;
    }

    return document.querySelector('paper-header-panel[' + selected + ']').scroller;
}

function scrollContentBy(x: number, y:number): void {
    let html_preview = document.getElementById('current-html-preview');
    if (html_preview) {
        html_preview.contentWindow.scrollBy(x, y);
    } else {
        // Note:
        // Scroll markdown preview

        let scroller = getScroller();
        if (!scroller) {
            return;
        }

        if (x !== 0) {
            scroller.scrollLeft += x;
        }
        if (y !== 0) {
            scroller.scrollTop += y;
        }
    }
}

window.onload = function(){
    const path = remote.require('./initial_path.js')();

    let Watcher = remote.require('./watcher.js');
    var watcher = new Watcher(
        path,

        // Markdown renderer
        function(kind: string, html: string): void {
            switch (kind) {
                case 'markdown': {
                    let markdown_preview = document.getElementById('current-markdown-preview');
                    if (markdown_preview !== null) {
                        markdown_preview.content = html;
                        return;
                    }

                    markdown_preview = document.createElement('markdown-preview');
                    markdown_preview.id = 'current-markdown-preview';

                    let wrapper = document.getElementById('viewer-wrapper');
                    if (wrapper.hasChildNodes()) {
                        wrapper.replaceChild(markdown_preview, wrapper.firstChild);
                    } else {
                        wrapper.appendChild(markdown_preview);
                    }

                    markdown_preview.content = html;
                    return;
                }

                case 'html': {
                    console.log(html);

                    // XXX: Temporary
                    // Now using 'div' container.  But I should use <webview> to load source properly
                    let html_preview = document.getElementById('current-html-preview');
                    if (html_preview !== null) {
                        html_preview.src = 'file://' + html;
                        return;
                    }

                    html_preview = document.createElement('iframe');

                    // html_preview = document.createElement('webview');
                    html_preview.id = 'current-html-preview';

                    let wrapper = document.getElementById('viewer-wrapper');
                    if (wrapper.hasChildNodes()) {
                        wrapper.replaceChild(html_preview, wrapper.firstChild);
                    } else {
                        wrapper.appendChild(html_preview);
                    }

                    html_preview.setAttribute('seamless', '');
                    html_preview.setAttribute('sandbox', 'allow-same-origin allow-top-navigation allow-forms allow-scripts');
                    html_preview.src = 'file://' + html;
                    return;
                }
            }
        },

        // Linter result renderer
        function(messages): void {
            document.getElementById('lint-area').content = messages;
            let button = document.getElementById('lint-button');
            if (messages.length === 0) {
                button.style.color = '#d99e5f';
            } else {
                button.style.color = '#ce3c4a';
            }
        }
    );

    document.getElementById('lint-area').lint_url = watcher.getLintRuleURL();

    let dialog = document.getElementById('path-change');
    dialog.path = path;
    dialog.onchanged = function(path) {
        watcher.changeWatchingDir(path);
        document.title = makeTitle(path);
    };

    if (path === '') {
        dialog.open();
    }
    document.title = makeTitle(path);

    KeyReceiver.on('Lint', function() {
            document.getElementById('main-drawer').togglePanel();
    });

    let scroller = document.querySelector('paper-header-panel[main]').scroller;

    KeyReceiver.on('PageUp', function() {
            scrollContentBy(0, -window.innerHeight / 2);
    });

    KeyReceiver.on('PageDown', function() {
            scrollContentBy(0, window.innerHeight / 2);
    });

    KeyReceiver.on('PageLeft', function() {
            scrollContentBy(-window.innerHeight / 2, 0);
    });

    KeyReceiver.on('PageRight', function() {
            scrollContentBy(window.innerHeight / 2, 0);
    });

    KeyReceiver.on('PageTop', function() {
            let scroller = getScroller();
            if (scroller) {
                scroller.scrollTop = 0;
            }
    });

    KeyReceiver.on('PageBottom', function() {
            let scroller = getScroller();
            if (scroller) {
                scroller.scrollTop = scroller.scrollHeight;
            }
    });

    KeyReceiver.on('ChangePath', function() {
            dialog.open();
    });
};
