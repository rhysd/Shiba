import * as globalShortcut from 'global-shortcut';

export default class KeyShortcuts {
    shortcuts: Object;

    constructor(browser_window, config) {
        let sender =  browser_window.webContents;
        const shortcuts = config.shortcuts;
        this.shortcuts = {};

        // Note: Generating below function in 'for' loop make jshint angry
        const key_receiver_for = function(s: string): () => void {
            return function() {
                sender.send('keyinput', s);
            };
        };

        for (const k in shortcuts) {
            var shortcut = shortcuts[k];

            if (!shortcut || shortcut === '') {
                continue;
            }

            if (shortcut === 'DevTools') {
                this.shortcuts[k] = () => browser_window.toggleDevTools();
                continue;
            }

            if (shortcut === 'QuitApp') {
                this.shortcuts[k] = () => browser_window.close();
                continue;
            }

            this.shortcuts[k] = key_receiver_for(shortcut);
        }

        browser_window.on('blur', () => {
            this.unregisterAll();
        });
        browser_window.on('focus', () => {
            this.registerAll();
        });
    }

    registerAll() {
        for (const key in this.shortcuts) {
            globalShortcut.register(key, this.shortcuts[key]);
        }
    }

    unregisterAll() {
        globalShortcut.unregisterAll();
    }
}

