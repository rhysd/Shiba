/// <reference path="lib.d.ts" />

namespace KeyReceiver {
    var ipc = require('ipc');
    var callbacks = {};

    ipc.on('keyinput', (event_name) => {
        if (event_name in callbacks) {
            callbacks[event_name]();
        }
    });

    export function on(event_name: string, callback: () => void) {
        callbacks[event_name] = callback;
    };
}
