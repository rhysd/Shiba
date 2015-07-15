'use strict';

let globalShortcut = require('global-shortcut');

function KeyShortcuts(browser_window, config) {
    let sender =  browser_window.webContents;
    const shortcuts = config.shortcuts;
    this.shortcuts = {};

    // Note: Generating below function in 'for' loop make jshint angry
    let quit_app = function() { browser_window.close(); };
    let toggle_devtools = function() { browser_window.toggleDevTools(); };

    for (const k in shortcuts) {
        const shortcut = shortcuts[k];

        if (!shortcut || shortcut === '') {
            continue;
        }

        if (shortcut === 'DevTools') {
            this.shortcuts[k] = toggle_devtools;
            continue;
        }

        if (shortcut === 'QuitApp') {
            this.shortcuts[k] = quit_app;
            continue;
        }

        this.shortcuts[k] = function() {
            sender.send('keyinput', shortcut);
        };
    }

    let that = this;
    browser_window.on('blur', function(){
        that.unregisterAll();
    });
    browser_window.on('focus', function(){
        that.registerAll();
    });
}

KeyShortcuts.prototype.registerAll = function() {
    for (const key in this.shortcuts) {
        globalShortcut.register(key, this.shortcuts[key]);
    }
};

KeyShortcuts.prototype.unregisterAll = function() {
    globalShortcut.unregisterAll();
};

module.exports = KeyShortcuts;
