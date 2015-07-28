/// <reference path="keyreceiver.ts" />
/// <reference path="lib.d.ts" />

var remote = require('remote');

function getPathDialog() {
    return <PathDialog>document.getElementById('path-change');
}

function getMainDrawerPanel() {
    return <MainDrawerPanel>document.getElementById('main-drawer');
}

function onPathButtonPushed(): void {
    getPathDialog().open();
}

function getLintArea() {
    return <LintResultArea>document.getElementById('lint-area');
}

function makeTitle(path: string): string {
    if (path === '') {
        return 'Shiba';
    } else {
        return 'Shiba (' + path + ')';
    }
}

function getScroller(): Scroller {
    const selected: string = getMainDrawerPanel().selected;
    if (selected === null) {
        return null;
    }

    return (<HeaderPanel>document.querySelector('paper-header-panel[' + selected + ']')).scroller;
}

function scrollContentBy(x: number, y:number): void {
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

function setChildToViewerWrapper(new_child: HTMLElement): void {
    let target = document.getElementById('viewer-wrapper');
    if (target.hasChildNodes()) {
        target.replaceChild(new_child, target.firstChild);
    } else {
        target.appendChild(new_child);
    }
}

function prepare_markdown_preview(html: string): void {
    let markdown_preview = <MarkdownPreview>document.getElementById('current-markdown-preview');
    if (markdown_preview !== null) {
        markdown_preview.content = html;
        return;
    }

    markdown_preview = <MarkdownPreview>document.createElement('markdown-preview');
    markdown_preview.id = 'current-markdown-preview';

    setChildToViewerWrapper(markdown_preview);

    markdown_preview.content = html;
}

function prepare_html_preview(file) {
    let html_preview = <HTMLIFrameElement>document.getElementById('current-html-preview');
    if (html_preview !== null) {
        html_preview.src = 'file://' + file;
        return;
    }

    html_preview = document.createElement('iframe');

    // html_preview = document.createElement('webview');
    html_preview.id = 'current-html-preview';
    html_preview.onload = function(e) {
        // Note:
        // Adjust
        html_preview.setAttribute('height', html_preview.contentWindow.document.body.scrollHeight + 'px');
    }

    html_preview.setAttribute('seamless', '');
    html_preview.setAttribute('sandbox', 'allow-same-origin allow-top-navigation allow-forms allow-scripts');
    html_preview.setAttribute('height', window.innerHeight + 'px');
    html_preview.src = 'file://' + file; // XXX: Escape double " and &

    setChildToViewerWrapper(html_preview);
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
                    prepare_markdown_preview(html);
                    return;
                }

                case 'html': {
                    prepare_html_preview(html);
                    return;
                }
            }
        },

        // Linter result renderer
        function(messages): void {
            getLintArea().content = messages;
            let button = document.getElementById('lint-button');
            if (messages.length === 0) {
                button.style.color = '#d99e5f';
            } else {
                button.style.color = '#ce3c4a';
            }
        }
    );

    getLintArea().lint_url = watcher.getLintRuleURL();

    let dialog = getPathDialog();
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
        getMainDrawerPanel().togglePanel();
    });

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
