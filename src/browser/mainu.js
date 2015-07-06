'use strict';

var app = require('app');
var path = require('path');
var ipc = require('ipc');
var shortcut = require('global-shortcut');
var BrowserWindow = require('browser-window');

require('crash-reporter').start();

// Main Window {{{
var mainWindow = null;

app.on('window-all-closed', function(){ app.quit(); });

app.on('ready', function(){
    mainWindow = new BrowserWindow(
        {
            icon: path.join('..', 'resource', 'image', 'shibainu.png'),
            width: 800,
            height: 600
        }
    );

    const html = 'file://' + path.resolve(__dirname, '..', '..', 'static', 'index.html');
    mainWindow.loadUrl(html);

    mainWindow.openDevTools();

    mainWindow.on('closed', function(){
        mainWindow = null;
        shortcut.unregisterAll();
    });
});
// }}}

