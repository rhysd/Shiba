import * as path from 'path';
import {app, BrowserWindow, shell} from 'electron';
import * as menu from './menu';
import {load as loadConfig} from './config';

const config = loadConfig();

// Show versions {{{
if (process.argv.indexOf('--version') !== -1) {
    const versions: any = process.versions;
    console.log(`Shiba: rich markdown previewer

Usage:
  $ shiba [--detach|--version] {directory to watch}

Versions:
  Shiba:    ${app.getVersion()}
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
    mainWindow = new BrowserWindow({
            icon: icon_path,
            width: getConfigLength('width', 800),
            height: getConfigLength('height', 600),
        } as Electron.BrowserWindowOptions);

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
});
// }}}

