/// <reference path="./keyboard.ts" />
/// <reference path="lib.d.ts" />

import * as path from 'path';
import * as fs from 'fs';
import {homedir} from 'os';
import {remote, ipcRenderer as ipc} from 'electron';
const config = remote.require('./config').load();
const home_dir = config.hide_title_bar ?  '' : homedir();

let watching_path = remote.require('./initial_path.js')();
let onPathButtonPushed = function(){ /* do nothing */ };
let onSearchButtonPushed = function(){ /* do nothing */ };
let onTOCButtonPushed = function(){ /* do nothing */ };

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

function make_title(p: string): string {
    if (!p || config.hide_title_bar) {
        return 'Shiba';
    }

    if (p.startsWith(home_dir)) {
        p = `~${p.slice(home_dir.length)}`;
    }

    return `Shiba (${p})`;
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

function openMarkdownDoc(file_path: string, modifier_key: boolean) {
    // Note:
    // This callback is called when open other document is opened.
    ipc.send('shiba:notify-path', file_path);
    if (modifier_key) {
        document.title = make_title(file_path);
        watching_path = file_path;
    }
}

function renderMarkdownPreview(file: string) {
    const exts = config.file_ext.markdown;
    const font_size = config.markdown.font_size;
    const isGitHubStyle = config.markdown.css_path.endsWith('/github-markdown.css');
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
            markdown_preview.fontSize = font_size;
        }
        if (!isGitHubStyle) {
            markdown_preview.isGithubStyle = false;
        }

        markdown_preview.exts = exts;
        markdown_preview.openMarkdownDoc = openMarkdownDoc;
        markdown_preview.onDocumentUpdated = () => getLintArea().showLintResult();
        setChildToViewerWrapper(markdown_preview);

        markdown_preview.document = markdown;
    });

    // Note:
    // This is a workaround to show linter result lazily.
    const lint = getLintArea();
    lint.already_previewed = false;
    lint.messages = undefined;
}

function renderHtmlPreview(file: string) {
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

function prepareMarkdownStyle(markdown_config: {
    css_path: string;
    code_theme: string;
}) {
    const {css_path, code_theme} = markdown_config;

    const markdown_css_link = document.createElement('link') as HTMLLinkElement;
    markdown_css_link.rel = 'stylesheet';
    markdown_css_link.href = css_path;
    document.head.appendChild(markdown_css_link);

    if (code_theme === '') {
        return;
    }

    const code_theme_css_link = document.createElement('link') as HTMLLinkElement;
    code_theme_css_link.rel = 'stylesheet';
    code_theme_css_link.href = `../../node_modules/highlight.js/styles/${code_theme}.css`;
    document.head.appendChild(code_theme_css_link);

    if (code_theme !== 'github' && css_path.endsWith('/github-markdown.css')) {
        console.warn('github-markdown.css overrides background color of code block.');
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

    prepareMarkdownStyle(config.markdown);

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
        if (!paths || paths.length === 0) {
            return '';
        }
        return paths[0];
    }

    ipc.on('shiba:notify-content-updated', (_: Event, kind: string, file: string) => {
        const reload_button = document.getElementById('reload-button');
        reload_button.classList.add('rotate');

        const base = document.querySelector('base');
        base.setAttribute('href', 'file://' + path.dirname(file) + path.sep);

        switch (kind) {
            case 'markdown': {
                renderMarkdownPreview(file);
                break;
            }
            case 'html': {
                renderHtmlPreview(file);
                break;
            }
            default:
                // Do nothing
                break;
        }
    });

    ipc.on('shiba:notify-linter-result', (_: Event, messages: LintMessage[]) => {
        lint.messages = messages;
        const button = document.getElementById('lint-button');
        if (messages.length === 0) {
            button.style.color = '#d99e5f';
        } else {
            button.style.color = '#ce3c4a';
        }
    });

    ipc.on('return-lint-url', (_: Event, url: string) => {
        lint.lint_url = url;
    });

    onPathButtonPushed = function() {
        const chosen = chooseFileOrDirWithDialog();
        if (chosen === '') {
            return;
        }
        watching_path = chosen;
        document.title = make_title(watching_path);
        ipc.send('shiba:notify-path', watching_path);
    };

    if (watching_path === '') {
        onPathButtonPushed();
    } else {
        ipc.send('shiba:notify-path', watching_path);
        document.title = make_title(watching_path);
    }

    const searcher = document.getElementById('builtin-page-searcher') as BuiltinSearch;
    onSearchButtonPushed = function() {
        searcher.toggle();
    };

    const toc = document.getElementById('table-of-contents') as TOCComponent;
    onTOCButtonPushed = function() {
        const preview = document.getElementById('current-markdown-preview') as MarkdownPreview;
        if (preview === null) {
            // TODO: Error handling
            return;
        }
        toc.toggle(preview.currentOutline);
    };
    toc.scrollCallback = function(h: Heading) {
        const preview = document.getElementById('current-markdown-preview') as MarkdownPreview;
        if (preview !== null) {
            preview.scrollToHeading(getScroller(), h);
        }
    };

    const cancel_event = function(e: Event) {
        e.preventDefault();
    };
    document.body.addEventListener('dragenter', cancel_event);
    document.body.addEventListener('dragover', cancel_event);
    document.body.addEventListener('drop', event => {
        event.preventDefault();
        const files = event.dataTransfer.files;
        if (files.length === 0) {
            return;
        }
        const p: string = (files[0] as any).path;
        if (!p) {
            console.log('Failed to get the path of dropped file');
            return;
        }
        watching_path = p;
        ipc.send('shiba:notify-path', p);
        document.title = make_title(p);
    });

    (document.querySelector('paw-filechooser') as PawFilechooser).onFileChosen = (file: string) => {
        watching_path = file;
        ipc.send('shiba:notify-path', file);
        document.title = make_title(file);
    };

    const reload_button = document.getElementById('reload-button');
    reload_button.onclick = () => renderMarkdownPreview(watching_path);
    reload_button.classList.add('animated');
    const reload_anime_listener = () => {
        reload_button.classList.remove('rotate');
    };
    reload_button.addEventListener('animationend', reload_anime_listener);

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
    receiver.on('Reload', () => renderMarkdownPreview(watching_path));
    receiver.on('Print', () => remote.getCurrentWindow().webContents.print());
    receiver.on('Search', () => onSearchButtonPushed());
    receiver.on('Outline', () => onTOCButtonPushed());

    searcher.onMount = () => { receiver.enabled = false; };
    searcher.onUnmount = () => { receiver.enabled = true; };
    toc.onMount = () => { receiver.enabled = false; };
    toc.onUnmount = () => { receiver.enabled = true; };

    ipc.on('shiba:choose-file', () => onPathButtonPushed());
    ipc.on('shiba:lint', () => getMainDrawerPanel().togglePanel());
    ipc.on('shiba:outline', () => onTOCButtonPushed());
    ipc.on('shiba:search', () => onSearchButtonPushed());
    ipc.on('shiba:reload', () => renderMarkdownPreview(watching_path));

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
