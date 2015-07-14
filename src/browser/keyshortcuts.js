'use strict';

let globalShortcut = require('global-shortcut');

function KeyShortcuts(browser_window, config) {
    let sender =  browser_window.webContents;
    const shortcuts = config.shortcuts;
    this.shortcuts = {};
    for (const k in shortcuts) {
        this.shortcuts[k] = function() {
            sender.send('keyinput', shortcuts[k]);
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
