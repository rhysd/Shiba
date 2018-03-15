/// <reference path="lib.d.ts" />

namespace Keyboard {
    export class Receiver {
        public enabled: boolean;
        private readonly callbacks: { [action: string]: () => void };

        constructor(private readonly shortcuts: { [k: string]: string }) {
            this.enabled = true;
            this.callbacks = {};

            const key_handler_for = (action: string) => () => this.dispatch_shortcut(action);
            for (const key of Object.keys(this.shortcuts)) {
                Mousetrap.bind(key, key_handler_for(this.shortcuts[key]));
            }
        }

        on(action: string, callback: () => void): void {
            this.callbacks[action] = callback;
        }

        dispatch_shortcut(action: string): void {
            if (this.enabled && action in this.callbacks) {
                this.callbacks[action]();
            }
        }
    }
}
