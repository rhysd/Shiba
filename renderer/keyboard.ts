/// <reference path="lib.d.ts" />

namespace Keyboard {
    'use strict';

    export class Receiver {
        private callbacks: {[action: string]: () => void};

        constructor(private shortcuts: {[k: string]: string}) {
            this.callbacks = {};

            const key_handler_for = (action: string) => () => this.dispatch_shortcut(action);
            for (const key in this.shortcuts) {
                Mousetrap.bind(key, key_handler_for(this.shortcuts[key]));
            }
        }

        on(action: string, callback: () => void): void {
            this.callbacks[action] = callback;
        }

        dispatch_shortcut(action: string): void {
            if (action in this.callbacks) {
                this.callbacks[action]();
            }
        }
    }
}
