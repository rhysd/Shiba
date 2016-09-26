declare namespace ElectronWindowState {
    interface WindowState {
        readonly x: number;
        readonly y: number;
        readonly width: number;
        readonly height: number;
        readonly isMaximized: boolean;
        readonly isFullScreen: boolean;
        manage(win: Electron.BrowserWindow): void;
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
