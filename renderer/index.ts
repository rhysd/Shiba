/// <reference path="./keyboard.ts" />
/// <reference path="lib.d.ts" />

import * as path from 'path';
import * as fs from 'fs';
import {remote, ipcRenderer as ipc} from 'electron';
const Watcher = remote.require('./watcher.js');
const config = remote.require('./config').load();

let watching_path = remote.require('./initial_path.js')();
let onPathButtonPushed = function(){ /* do nothing */ };
let onSearchButtonPushed = function(){ /* do nothing */ };

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

    if (selected === 'drawer') {
        const panel = document.querySelector('paper-header-panel[drawer]') as HeaderPanel;
        return panel.scroller;
    } else {
        return document.getElementById('viewer-wrapper');
    }
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

function prepareMarkdownPreview(file: string, exts: string[], font_size: string, onPathChanged: (p: string, m: boolean) => void): void {
    fs.readFile(file, 'utf8', (err: Error, markdown: string) => {
        if (err) {
            console.error(err);
            return;
        }

        let markdown_preview = document.getElementById('current-markdown-preview') as MarkdownPreview;
        if (markdown_preview !== null) {
            markdown_preview.document = markdown;
            return;
        }

        markdown_preview = document.createElement('markdown-preview') as MarkdownPreview;
        markdown_preview.id = 'current-markdown-preview';
        if (font_size !== '') {
            markdown_preview.setAttribute('font-size', font_size);
        }

        setChildToViewerWrapper(markdown_preview);

        markdown_preview.exts = exts;
        markdown_preview.openMarkdownDoc = onPathChanged;
        markdown_preview.document = markdown;
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

function getDialogDefaultPath() {
    try {
        const stats = fs.lstatSync(watching_path);
        if (stats.isDirectory()) {
            return watching_path;
        }
        return path.dirname(watching_path);
    } catch (e) {
        // Note: Path not found
        return '';
    }
}

(function(){
    const lint = getLintArea();
    if (config.voice.enabled) {
        lint.voice_src = config.voice.source;
    }
    if (config.hide_title_bar) {
        lint.enable_inset = config.hide_title_bar;
    }

    function chooseFileOrDirWithDialog() {
        const filters = [
            {
                name: 'Markdown',
                extensions: config.file_ext.markdown,
            },
            {
                name: 'HTML',
                extensions: config.file_ext.html,
            },
        ];
        const paths = remote.dialog.showOpenDialog({
            title: 'Choose file or directory to watch',
            defaultPath: getDialogDefaultPath(),
            filters,
            properties: ['openFile', 'openDirectory'],
        });
        console.log(paths);
        if (!paths || paths.length === 0) {
            return '';
        }
        return paths[0];
    }

    const watcher = new Watcher(
        watching_path,

        // Markdown renderer
        function(kind: string, file: string): void {
            const base = document.querySelector('base');
            base.setAttribute('href', 'file://' + path.dirname(file) + path.sep);
            switch (kind) {
                case 'markdown': {
                    prepareMarkdownPreview(file, config.file_ext.markdown, config.markdown.font_size, (file_path: string, modifier: boolean) => {
                        if (modifier) {
                            watcher.changeWatchingDir(file_path);
                            document.title = makeTitle(file_path);
                        } else {
                            watcher.sendUpdate(file_path);
                        }
                    });
                }
                break;

                case 'html': {
                    prepareHtmlPreview(file);
                }
                break;

                default: {
                    // Do nothing
                }
                break;
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
        const chosen = chooseFileOrDirWithDialog();
        if (chosen === '') {
            return;
        }
        watching_path = chosen;
        document.title = makeTitle(watching_path);
        watcher.changeWatchingDir(watching_path);
    };

    if (watching_path === '') {
        onPathButtonPushed();
    }

    const searcher = document.getElementById('builtin-page-searcher') as BuiltinSearch;

    onSearchButtonPushed = function() {
        searcher.toggle();
    };

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

    const menu = document.getElementById('menu');
    if (!config.menu.visible) {
        menu.style.display = 'none';
    } else if (config.hide_title_bar && process.platform === 'darwin') {
        const spacer = document.getElementById('inset-spacer');
        spacer.style.height = '25px';
        // Note:
        // Tweak width of menu bar.
        // Width of traffic lights gets wider when a title bar is hidden.
        menu.style.width = '80px';
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
        this.bw.getFocusedWindow().openDevTools({detach: true});
    });
    receiver.on('Reload', () => watcher.startWatching());
    receiver.on('Print', () => remote.getCurrentWindow().webContents.print());
    receiver.on('Search', () => searcher.toggle());

    searcher.onMount = () => { receiver.enabled = false; };
    searcher.onUnmount = () => { receiver.enabled = true; };

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
