declare function require(mod: string): any; // tsurai

interface Scroller {
    scrollLeft: number;
    scrollTop: number;
    scrollHeight: number;
}

interface PathDialog extends HTMLElement {
    open(): void;
    path: string;
    onchanged: (string) => void;
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
}

interface LintResultArea extends HTMLElement {
    content: string;
    lint_url: string;
    voice_src: string;
}

