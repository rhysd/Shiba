import * as app from 'app';
import * as path from 'path';
import * as fs from 'fs';
import BrowserWindow = require('browser-window');
import {openExternal} from 'shell';
import * as menu from './menu';
import {load as loadConfig} from './config';

require('crash-reporter').start();

const config = loadConfig();

// Show versions {{{
const versions: any = process.versions;
console.log('Shiba version 0.3.9');
console.log('  Electron version ' + versions.electron);
console.log('  Chrome version ' + versions.chrome);
console.log('  io.js version ' + versions.node);
// }}}

// Main Window {{{
var mainWindow = null;

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

    mainWindow = new BrowserWindow(
        {
            icon: path.join(__dirname, '..', '..', 'images', 'shibainu.png'),
            width: getConfigLength('width', 800),
            height: getConfigLength('height', 600)
        }
    );

    const html = 'file://' + path.resolve(__dirname, '..', '..', 'static', 'index.html');
    mainWindow.loadUrl(html);

    mainWindow.on('closed', function(){
        mainWindow = null;
    });

    mainWindow.on('will-navigate', function(e: Event, url: string){
        e.preventDefault();
        openExternal(url);
    });

    mainWindow.on('dom-ready', function(){
        fs.readFile(path.join(app.getPath('userData'), 'user.css'), {encoding: 'utf8'}, (err, content) => {
            if (err) {
                return;
            }

            mainWindow.webContents.insertCSS(content);
        });
    });

    menu.build(mainWindow);
});
// }}}

