import windowState = require('electron-window-state');
import * as path from 'path';
import {app, BrowserWindow, screen} from 'electron';
import loadAppConfig from './config';
import Doghouse from './doghouse';
import Ipc from './ipc';
import log from './log';

process.on('unhandledRejection', (reason: string, p: Promise<any>) => {
    log.error('FATAL: Unhandled rejection at Promise', p, ', Reason:', reason);
});

let win = null as (Electron.BrowserWindow | null);

function isRunFromNpmPackageOnDarwin() {
    return process.platform === 'darwin' && app.getAppPath().indexOf('/Shiba.app/') === -1;
}

function getWindowSize(config: AppConfig): {width: number, height: number} {
    const display_size = screen.getPrimaryDisplay().workAreaSize;
    const size = {
        height: 800,
        width: 600,
    };
    if (typeof config.height === 'number') {
        size.height = config.height;
    } else {
        size.height = display_size.height;
    }
    if (typeof config.width === 'number') {
        size.width = config.width;
    } else {
        size.width = display_size.width;
    }
    return size;
}

function openWindow(config: AppConfig) {
    log.debug('Starting window with config', config);
    return new Promise<[Electron.BrowserWindow, AppConfig]>(resolve => {
        const config_size = getWindowSize(config);
        const window_state = windowState({
            defaultWidth: config_size.width,
            defaultHeight: config_size.height,
        });
        const icon_path = path.join(__dirname, '..', 'images', 'icon', 'icon.png');
        if (config.restore_window_state !== false) {
            win = new BrowserWindow({
                x: window_state.x,
                y: window_state.y,
                width: window_state.width,
                height: window_state.height,
                titleBarStyle: config.hide_title_bar ? 'hidden-inset' : 'default',
                autoHideMenuBar: !!config.hide_menu_bar,
                icon: icon_path,
            });
            if (window_state.isFullScreen) {
                win.setFullScreen(true);
            } else if (window_state.isMaximized) {
                win.maximize();
            }
            log.debug('Restored window state', window_state);
            window_state.manage(win);
        } else {
            win = new BrowserWindow({
                width: config_size.width,
                height: config_size.height,
                titleBarStyle: config.hide_title_bar ? 'hidden-inset' : 'default',
                autoHideMenuBar: !!config.hide_menu_bar,
                icon: icon_path,
            });
            log.debug('Created window without restoring state', config_size);
        }

        win.once('closed', () => { win = null; });

        win.webContents.on('dom-ready', () => {
            log.debug('DOM ready');
            resolve([win, config]);
        });

        const index_html = 'file://' + path.join(__dirname, '..', 'renderer', 'index.html');
        win.loadURL(index_html);
        log.debug('Loaded HTML to window', index_html);

        if (isRunFromNpmPackageOnDarwin()) {
            app.dock.setIcon(icon_path);
            log.debug('Icon was set to dock', icon_path);
        }

        if (process.env.NODE_ENV === 'development') {
            win.webContents.once('devtools-opened', () => setImmediate(() => win && win.focus()));
            win.webContents.openDevTools({mode: 'detach'});
            log.debug('DevTools was opened in background');
        }
    });
}

function setupDoghouse([w, c]: [Electron.BrowserWindow, AppConfig]) {
    const doghouse = new Doghouse(c);
    Ipc.onReceive('shiba:tab-opened', (p: string) => {
        doghouse.newDog(p).then(dog => new Ipc(dog, w.webContents));
    });
    Ipc.onReceive('shiba:tab-closed', (id: number) => {
        doghouse.removeDog(id);
        // Note: Should send FIN to renderer?
    });
}

app.once('window-all-closed', () => app.quit());
app.on('activate', () => {
    log.debug('Application was activated');
    if (win) {
        win.show();
    }
});
app.once('ready', () => loadAppConfig().then(openWindow).then(setupDoghouse));
