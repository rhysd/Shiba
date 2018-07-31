/// <reference lib="es2015.promise" />

interface Set<T> {
    has(value: T): boolean;
}

interface SetConstructor {
    prototype: Set<any>;
    new <T>(iterable?: any[]): Set<T>;
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
    onSanitizationError: (message: string, reason: string) => void;
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
    sanitize_error: LintMessage;
    showLintResult(): void;
}

interface LintMessageElement extends HTMLElement {
    header: string;
    body: string;
}

interface PawFilechooser extends HTMLElement {
    onFileChosen: (path: string) => void;
}

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
    selectionArea: object;
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
