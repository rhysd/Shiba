/// <reference path="./keyboard.ts" />
/// <reference path="./emoji.ts" />
/// <reference path="lib.d.ts" />

import * as path from 'path';
import * as fs from 'fs';
import * as marked from 'marked';
import * as katex from 'katex';
import {highlight} from 'highlight.js';
import {remote, ipcRenderer as ipc} from 'electron';
const Watcher = remote.require('./watcher.js');
const config = remote.require('./config').load();

let current_path = remote.require('./initial_path.js')();
let onPathButtonPushed = function(){ /* do nothing */ };
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

function getMainDrawerPanel() {
    return <MainDrawerPanel>document.getElementById('main-drawer');
}

/* tslint:disable no-unused-variable*/
function onPrintButtonPushed(): void {
    remote.getCurrentWindow().webContents.print();
}
/* tslint:enable no-unused-variable*/

function getLintArea() {
    return document.getElementById('lint-area') as LintResultArea;
}

function makeTitle(p: string): string {
    if (p === '') {
        return 'Shiba';
    } else {
        return `Shiba (${p})`;
    }
}

function getScroller(): Scroller {
    const selected: string = getMainDrawerPanel().selected;
    if (selected === null) {
        return null;
    }

    const panel = document.querySelector('paper-header-panel[' + selected + ']') as HeaderPanel;
    return panel.scroller;
}

function scrollContentBy(x: number, y: number) {
    const scroller = getScroller();
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
    const target = document.getElementById('viewer-wrapper');
    if (target.hasChildNodes()) {
        target.replaceChild(new_child, target.firstChild);
    } else {
        target.appendChild(new_child);
    }
}

function prepareMarkdownPreview(file: string, exts: string[], onPathChanged: (p: string, m: boolean) => void): void {
    fs.readFile(file, 'utf8', (err: Error, markdown: string) => {
        if (err) {
            console.error(err);
            return;
        }

        const html = MarkdownRenderer.render(markdown);

        let markdown_preview = document.getElementById('current-markdown-preview') as MarkdownPreview;
        if (markdown_preview !== null) {
            markdown_preview.content = html;
            return;
        }

        markdown_preview = document.createElement('markdown-preview') as MarkdownPreview;
        markdown_preview.id = 'current-markdown-preview';

        setChildToViewerWrapper(markdown_preview);

        markdown_preview.exts = exts;
        markdown_preview.openMarkdownDoc = onPathChanged;
        markdown_preview.content = html;
    });
}

function prepareHtmlPreview(file: string) {
    let html_preview = document.getElementById('current-html-preview') as HTMLIFrameElement;
    if (html_preview !== null) {
        html_preview.src = 'file://' + file;
        return;
    }

    html_preview = document.createElement('iframe');

    // html_preview = document.createElement('webview');
    html_preview.id = 'current-html-preview';
    html_preview.className = 'current-html-preview';
    html_preview.onload = function(e) {
        // Note:
        // Adjust
        html_preview.setAttribute('height', html_preview.contentWindow.document.body.scrollHeight + 'px');
    };

    html_preview.setAttribute('seamless', '');
    html_preview.setAttribute('sandbox', 'allow-same-origin allow-top-navigation allow-forms allow-scripts');
    html_preview.setAttribute('height', window.innerHeight + 'px');
    html_preview.src = 'file://' + file; // XXX: Escape double " and &

    setChildToViewerWrapper(html_preview);
}

(function(){
    const lint = getLintArea();
    if (config.voice.enabled) {
        lint.voice_src = config.voice.source;
    }

    function chooseFileOrDirWithDialog() {
        // TODO: Filter by extentions
        const paths = remote.dialog.showOpenDialog({
            title: 'Choose file or directory to watch',
            defaultPath: current_path,
            filters: [
                {
                    name: 'Markdown',
                    extensions: ['md', 'markdown', 'mkd'],
                },
                {
                    name: 'HTML',
                    extensions: ['html'],
                },
            ],
            properties: ['openFile', 'openDirectory'],
        });
        console.log(paths);
        if (!paths || paths.length === 0) {
            return '';
        }
        return paths[0];
    }

    const watcher = new Watcher(
        current_path,

        // Markdown renderer
        function(kind: string, file: string): void {
            const base = document.querySelector('base');
            base.setAttribute('href', 'file://' + path.dirname(file) + path.sep);
            switch (kind) {
                case 'markdown': {
                    prepareMarkdownPreview(file, config.file_ext.markdown, (file_path: string, modifier: boolean) => {
                        if (modifier) {
                            watcher.changeWatchingDir(file_path);
                            document.title = makeTitle(file_path);
                        } else {
                            watcher.sendUpdate(file_path);
                        }
                    });
                    return;
                }

                case 'html': {
                    prepareHtmlPreview(file);
                    return;
                }

                default: {
                    // Do nothing
                    break;
                }
            }
        },

        // Linter result renderer
        function(messages: LintMessage[]): void {
            lint.content = messages;
            const button = document.getElementById('lint-button');
            if (messages.length === 0) {
                button.style.color = '#d99e5f';
            } else {
                button.style.color = '#ce3c4a';
            }
        }
    );

    lint.lint_url = watcher.getLintRuleURL();

    onPathButtonPushed = function() {
        current_path = chooseFileOrDirWithDialog();
        document.title = makeTitle(current_path);
        watcher.changeWatchingDir(current_path);
    };

    if (current_path === '') {
        onPathButtonPushed();
    }

    const cancel_event = function(e: Event) {
        e.preventDefault();
    };
    document.body.addEventListener('dragenter', cancel_event);
    document.body.addEventListener('dragover', cancel_event);
    document.body.addEventListener('drop', event => {
        event.preventDefault();
        const file: any = event.dataTransfer.files[0];
        if (file === undefined) {
            return;
        }
        // XXX: `path` is not standard member of `File` class
        if (file.path === undefined) {
            console.log('Failed to get the path of dropped file');
            return;
        }
        watcher.changeWatchingDir(file.path);
        document.title = makeTitle(file.path);
    });

    (<PawFilechooser>document.querySelector('paw-filechooser')).onFileChosen = (file: string) => {
        watcher.changeWatchingDir(file);
        document.title = makeTitle(file);
    };

    const reload_button = document.getElementById('reload-button');
    reload_button.onclick = () => watcher.startWatching();

    if (!config.drawer.responsive) {
        const drawer: any = document.getElementById('main-drawer');
        drawer.forceNarrow = true;
    }

    if (!config.menu.visible) {
        const menu = document.getElementById('menu');
        menu.style.display = 'none';
    }

    const receiver = new Keyboard.Receiver(config.shortcuts);

    receiver.on('Lint', () => getMainDrawerPanel().togglePanel());
    receiver.on('PageUp', () => scrollContentBy(0, -window.innerHeight / 2));
    receiver.on('PageDown', () => scrollContentBy(0, window.innerHeight / 2));
    receiver.on('PageLeft', () => scrollContentBy(-window.innerHeight / 2, 0));
    receiver.on('PageRight', () => scrollContentBy(window.innerHeight / 2, 0));
    receiver.on('ChangePath', () => onPathButtonPushed());
    receiver.on('QuitApp', () => remote.app.quit());
    receiver.on('PageTop', () => {
        const scroller = getScroller();
        if (scroller) {
            scroller.scrollTop = 0;
        }
    });
    receiver.on('PageBottom', () => {
        const scroller = getScroller();
        if (scroller) {
            scroller.scrollTop = scroller.scrollHeight;
        }
    });
    receiver.on('DevTools', function() {
        this.bw = this.bw || remote.BrowserWindow;
        this.bw.getFocusedWindow().toggleDevTools();
    });
    receiver.on('Reload', () => watcher.startWatching());
    receiver.on('Print', () => remote.getCurrentWindow().webContents.print());

    ipc.on('shiba:choose-file', () => onPathButtonPushed());
    ipc.on('shiba:lint', () => getMainDrawerPanel().togglePanel());
    ipc.on('shiba:reload', () => watcher.startWatching());

    const user_css_path: string = path.join(config._config_dir_path, 'user.css');
    fs.exists(user_css_path, (exists: boolean) => {
        if (!exists) {
            return;
        }

        const link = document.createElement('link');
        link.rel = 'stylesheet';
        link.type = 'text/css';
        link.href = 'file://' + user_css_path;
        document.head.appendChild(link);
    });
})();
