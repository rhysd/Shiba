/// <reference path="../typings/browser.d.ts" />

interface Scroller {
    scrollLeft: number;
    scrollTop: number;
    scrollHeight: number;
}

interface PathDialog extends HTMLElement {
    open(): void;
    close(): void;
    path: string;
    onchanged: (path: string) => void;
}

interface MainDrawerPanel extends HTMLElement {
    selected: any; // tsurai
    togglePanel: () => void;
}

interface HeaderPanel extends HTMLElement {
    scroller: Scroller;
}

interface MarkdownPreview extends HTMLElement {
    content: string;
    exts: string[];
    openMarkdownDoc: (path: string, modifier: boolean) => void;
}

interface LintMessage {
    header: string;
    body: string;
}

interface LintResultArea extends HTMLElement {
    content: LintMessage[];
    lint_url: string;
    voice_src: string;
}

interface LintMessageElement extends HTMLElement {
    header: string;
    body: string;
}

interface PawFilechooser extends HTMLElement {
    onFileChosen: (path: string) => void;
}

declare var mermaid: any;

interface String {
    startsWith(needle: string): boolean;
    endsWith(needle: string): boolean;
}
