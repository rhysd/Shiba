/// <reference path="./config.d.ts" />

declare namespace NodeJS {
    interface Global {
        config: Config;
    }
}

interface String {
    endsWith(s: string): boolean;
}

declare module 'markdownlint' {
    const lint: (opts: Object, callback: (err: Error, result: any) => void) => void;
    export = lint;
}

declare namespace RemarkLint {
    interface Linter { /* TODO */ }
}

declare module 'remark-lint' {
    const remarklint: RemarkLint.Linter;
    export = remarklint;
}

declare namespace Remark {
    interface Processor {
        use(plugin: RemarkLint.Linter, opts: Object): Processor;
        process(content: string, callback: (err: Error, file: any) => void): void;
    }
}

declare module 'remark' {
    const remark: () => Remark.Processor;
    export = remark;
}

interface LinterMessage {
    header: string;
    body: string;
    line?: number;
    column?: number;
}

declare namespace ElectronWindowState {
    interface WindowState {
        x: number;
        y: number;
        width: number;
        height: number;
        isMaximized: boolean;
        isFullScreen: boolean;
        manage(win: Electron.BrowserWindow): void;
        unmanage(): void;
        saveState(win: Electron.BrowserWindow): void;
    }
    interface WindowStateKeeperOptions {
        defaultWidth?: number;
        defaultHeight?: number;
        path?: string;
        file?: string;
        maximize?: boolean;
        fullScreen?: boolean;
    }
}

declare module 'electron-window-state' {
    function windowStateKeeper(opts: ElectronWindowState.WindowStateKeeperOptions): ElectronWindowState.WindowState;
    export = windowStateKeeper;
}

interface String {
    startsWith(needle: string): boolean;
    endsWith(needle: string): boolean;
    repeat(count: number): string;
}

