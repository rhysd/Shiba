import * as path from 'path';
import {app, BrowserWindow, shell} from 'electron';
import * as menu from './menu';
import {load as loadConfig} from './config';

const config = loadConfig();

// Show versions {{{
if (process.argv.indexOf('--version') !== -1) {
    const versions: any = process.versions;
    console.log(`Shiba v${app.getVersion()}: Rich markdown previewer

Usage:
  $ shiba [--detach|--version] [{direcotyr/file to watch}]

Environment:
  OS:       ${process.platform}-${process.arch}
  Electron: ${versions.electron}
  Chrome:   ${versions.chrome}
  Node.js:  ${versions.node}
`);
    app.quit();
}
// }}}

// Main Window {{{
let mainWindow: Electron.BrowserWindow = null;

app.on('window-all-closed', function(){ app.quit(); });

app.on('ready', function(){
    const display_size = require('screen').getPrimaryDisplay().workAreaSize;

    function getConfigLength(key: string, default_len: number): number {
        const len = config[key];
        switch (typeof len) {
            case 'string': {
                if (len === 'max') {
                    return display_size[key];
                }
                return default_len;
            }
            case 'number': {
                return len;
            }
            default: {
                return default_len;
            }
        }
    }

    const icon_path = path.join(__dirname, '..', '..', 'images', 'shibainu.png');
    const options: Electron.BrowserWindowOptions = {
            icon: icon_path,
            width: getConfigLength('width', 920),
            height: getConfigLength('height', 800),
        };
    if (config.hide_title_bar) {
        options.titleBarStyle = 'hidden-inset';
    }
    mainWindow = new BrowserWindow(options);

    const html = 'file://' + path.resolve(__dirname, '..', '..', 'static', 'index.html');
    mainWindow.loadURL(html);

    mainWindow.on('closed', function(){
        mainWindow = null;
    });

    mainWindow.on('will-navigate', function(e: Event, url: string){
        e.preventDefault();
        shell.openExternal(url);
    });

    menu.build(mainWindow);

    if (process.argv[0].endsWith('Electron') && process.platform === 'darwin') {
        // If Shiba is run as npm package, replace dock app icon
        app.dock.setIcon(icon_path);
    }

    if (process.env.NODE_ENV === 'development') {
        mainWindow.webContents.on('devtools-opened', () => setImmediate(() => mainWindow.focus()));
        mainWindow.webContents.openDevTools({detach: true});
    }
});
// }}}

