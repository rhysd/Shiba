'use strict';

let ipc = require('ipc');

function KeyReceiver() {
    this.callbacks = {};

    let that = this;
    ipc.on('keyinput', function(event_name){
        if (event_name in that.callbacks) {
            that.callbacks[event_name]();
        }
    });
}

KeyReceiver.prototype.on = function(event_name, callback) {
    this.callbacks[event_name] = callback;
};
