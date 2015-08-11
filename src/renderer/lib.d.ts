declare function require(mod: string): any; // tsurai

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

interface LintResultArea extends HTMLElement {
    content: string;
    lint_url: string;
    voice_src: string;
}

interface LintMessage {
    header: string;
    body: string;
}

interface LintMessageElement extends HTMLElement {
    message: LintMessage;
}

interface String {
    startsWith(needle: string): boolean;
    endsWith(needle: string): boolean;
}
