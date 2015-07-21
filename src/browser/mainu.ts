import app = require('app');
import path = require('path');
import ipc = require('ipc');
import BrowserWindow = require('browser-window');
import menu = require('./menu');
const config = require('./config').load();
import KeyShortcuts = require('./keyshortcuts');

require('crash-reporter').start();

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
            icon: path.join(__dirname, '..', '..', 'image', 'shibainu.png'),
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

