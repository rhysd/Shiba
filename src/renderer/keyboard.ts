/// <reference path="lib.d.ts" />
/// <reference path="../../typings/mousetrap/mousetrap.d.ts" />

namespace Keyboard {

    export class Receiver {
        private callbacks: Object;

        constructor(private shortcuts) {
            this.callbacks = {};

            const key_handler_for = action => () => this.dispatch(action);
            for (const key in this.shortcuts) {
                Mousetrap.bind(key, key_handler_for(this.shortcuts[key]));
            }
        }

        on(action: string, callback: () => void): void {
            this.callbacks[action] = callback;
        }

        dispatch(action: string): void {
            if (action in this.callbacks) {
                this.callbacks[action]();
            }
        }
    }
}
