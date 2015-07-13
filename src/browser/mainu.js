'use strict';

let app = require('app');
let path = require('path');
let ipc = require('ipc');
let shortcut = require('global-shortcut');
let BrowserWindow = require('browser-window');
let menu = require('./menu.js');
const config = require('./config.js').load();

require('crash-reporter').start();

// Main Window {{{
var mainWindow = null;

app.on('window-all-closed', function(){ app.quit(); });

app.on('ready', function(){
    const display_size = require('screen').getPrimaryDisplay().workAreaSize;

    function getConfigLength(key, default_len) {
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
            icon: path.join('..', 'resource', 'image', 'shibainu.png'),
            width: getConfigLength('width', 800),
            height: getConfigLength('height', 600)
        }
    );

    const html = 'file://' + path.resolve(__dirname, '..', '..', 'static', 'index.html');
    mainWindow.loadUrl(html);

    mainWindow.on('closed', function(){
        mainWindow = null;
        shortcut.unregisterAll();
    });

    menu.build(mainWindow);
});
// }}}

