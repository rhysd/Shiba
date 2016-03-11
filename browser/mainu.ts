import * as path from 'path';
import {app, BrowserWindow, shell} from 'electron';
import * as menu from './menu';
import {load as loadConfig} from './config';

const config = loadConfig();

// Show versions {{{
const versions: any = process.versions;
console.log('Shiba version ' + app.getVersion());
console.log('  Electron version ' + versions.electron);
console.log('  Chrome version ' + versions.chrome);
console.log('  Node.js version ' + versions.node);
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

    mainWindow = new BrowserWindow({
            icon: path.join(__dirname, '..', '..', 'images', 'shibainu.png'),
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
});
// }}}

