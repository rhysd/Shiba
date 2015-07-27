declare function require(mod: string): any; // tsurai

interface Scroller {
    scrollLeft: number;
    scrollTop: number;
    scrollHeight: number;
}

interface HTMLElement {
    open(): void;
    selected: any; // tsurai
    scroller: Scroller;
    content: string;
    lint_url: string;
    path: string;
    onchanged: (string) => void;
    togglePanel: () => void;
    src: string;
    contentWindow: typeof window;
}

interface Element {
    scroller: Scroller;
    content: string;
}
