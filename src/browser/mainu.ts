import * as app from 'app';
import * as path from 'path';
import BrowserWindow = require('browser-window');
import {openExternal} from 'shell';
import * as menu from './menu';
import KeyShortcuts from './keyshortcuts';
import {load as loadConfig} from './config';

require('crash-reporter').start();

const config = loadConfig();

// Main Window {{{
var mainWindow = null;

app.on('window-all-closed', function(){ app.quit(); });

app.on('open-url', function(event){
    event.preventDefault();
    console.log('Tend to open: ' + event.url);
    openExternal(event.url);
});

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

    let keyshortcuts = new KeyShortcuts(mainWindow, config);

    mainWindow.on('closed', function(){
        keyshortcuts.unregisterAll();
        mainWindow = null;
    });

    menu.build(mainWindow);
});
// }}}

