interface Set<T> {
    has(value: T): boolean;
}

interface SetConstructor {
    new <T>(): Set<T>;
    new <T>(iterable: any[]): Set<T>;
    prototype: Set<any>;
}
declare var Set: SetConstructor;

interface Scroller {
    scrollLeft: number;
    scrollTop: number;
    scrollHeight: number;
}

interface MainDrawerPanel extends HTMLElement {
    selected: any; // tsurai
    togglePanel: () => void;
}

interface HeaderPanel extends HTMLElement {
    scroller: Scroller;
}

interface Heading {
    title: string;
    hash: string;
    level: number;
    html: string;
}

interface MarkdownPreview extends HTMLElement {
    document: string;
    exts: string[];
    fontSize: string;
    currentOutline: Heading[];
    isGithubStyle: boolean;
    openMarkdownDoc: (path: string, modifier: boolean) => void;
    onDocumentUpdated: () => void;
    scrollToHeading(e: Scroller, h: Heading): void;
}

interface LintMessage {
    header: string;
    body: string;
    line?: number;
    column?: number;
}

interface LintResultArea extends HTMLElement {
    messages: LintMessage[];
    lint_url: string;
    voice_src: string;
    enable_inset: boolean;
    already_previewed: boolean;
    showLintResult(): void;
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
    repeat(count: number): string;
}

interface FoundInPage {
    requestId: number;
    finalUpdate: boolean;
    activeMatchOrdinal?: number;
    matches?: number;
    selectionArea: Object;
}

interface BuiltinSearch extends HTMLElement {
    displayed: boolean;
    searching: boolean;
    activeIdx: number;
    onMount: () => void;
    onUnmount: () => void;
    show(): void;
    dismiss(): void;
    toggle(): void;
    search(text: string): void;
    searchNext(text: string, forward: boolean): void;
    stopSearch(): void;
    setResult(no: number, all: number): void;
}

interface MarkdownPreviewComponent extends polymer.Base {
    _documentUpdated(markdown_doc: string): void;
}

interface PaperDialogElement {
    open(): void;
    close(): void;
}

interface TOCComponent extends HTMLElement {
    opened: boolean;
    innerDialog: PaperDialogElement;
    currentItems: HTMLElement[];
    selectedIdx: number;
    currentOutline: Heading[];
    scrollCallback: (h: Heading) => void;
    onMount: () => void;
    onUnmount: () => void;
    open(): void;
    close(): void;
    toggle(outline?: Heading[]): void;
    focusNext(): void;
    focusPrevious(): void;
    copyOutlineToClipboard(): void;
}

interface WebViewElement extends HTMLElement {
    src: string;
}
