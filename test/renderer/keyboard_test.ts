/// <reference path="lib.d.ts" />
/// <reference path="../../renderer/keyboard.ts" />

import {runInThisContext} from 'vm';
import {readFileSync} from 'fs';
import {join} from 'path';
import * as assert from 'power-assert';

// Stub
type Handler = () => void;
class MousetrapStub {
    public handlers: {[k: string]: Handler};

    constructor() {
        this.handlers = {};
    }

    bind(key: string, handler: Handler) {
        this.handlers[key] = handler;
    }

    reset() {
        this.handlers = {};
    }

    dispatch(key: string) {
        if (this.handlers[key]) {
            this.handlers[key]();
            return true;
        } else {
            return false;
        }
    }
}

const stub = new MousetrapStub();
(global as any).Mousetrap = stub; // To inject to keyboard.ts

const dependant_script = join(__dirname, '..', '..', '..', '..', 'build', 'src', 'renderer', 'keyboard.js');
runInThisContext(readFileSync(dependant_script, 'utf8'));

describe('Keyboard.Receiver', () => {
    afterEach(() => {
        stub.reset();
    });

    describe('#constructor()', () => {
        it('registers given shortcuts', () => {
            const shortcuts = {
                j:        'PageDown',
                k:        'PageUp',
                down:     'PageDown',
                up:       'PageUp',
            } as {[k: string]: string};
            const r = new Keyboard.Receiver(shortcuts);
            assert(r);
            assert(Object.keys(stub.handlers).length === 4);
            assert('j' in stub.handlers);
            assert('k' in stub.handlers);
            assert('down' in stub.handlers);
            assert('up' in stub.handlers);
        });

        it('enables itself by default', () => {
            const r = new Keyboard.Receiver({});
            assert(r.enabled);
        });
    });

    describe('#on()', () => {
        it('registers a handler when specific action is fired', () => {
            const shortcuts = {
                j: 'Action1',
                k: 'Action2',
            } as {[k: string]: string};

            const r = new Keyboard.Receiver(shortcuts);

            let flag1 = false;
            r.on('Action1', () => {
                flag1 = true;
            });
            let flag2 = false;
            r.on('Action2', () => {
                flag2 = true;
            });
            assert(stub.dispatch('j'));
            assert(flag1);
            assert(!flag2);
        });
    });

    describe('#dispatch_shortcut()', () => {
        it('dispatches a handler related specified action', () => {
            const shortcuts = {
                j: 'Action1',
                k: 'Action2',
            } as {[k: string]: string};

            const r = new Keyboard.Receiver(shortcuts);

            let flag = false;
            r.on('Action1', () => {
                flag = true;
            });
            r.dispatch_shortcut('Action1');
            assert(flag);
        });

        it('does not fire no handler on unknown action', () => {
            const shortcuts = {
                j: 'Action1',
                k: 'Action2',
            } as {[k: string]: string};

            const r = new Keyboard.Receiver(shortcuts);

            let flag = false;
            r.on('Action1', () => {
                flag = true;
            });
            r.dispatch_shortcut('UnknownAction');
            assert(!flag);
        });
    });

    describe('#enabled', () => {
        it('prevent handlers from being called if false', () => {
            const shortcuts = {
                j: 'Action1',
                k: 'Action2',
            } as {[k: string]: string};

            const r = new Keyboard.Receiver(shortcuts);
            let flag = false;

            r.enabled = false;

            r.on('Action1', () => {
                flag = true;
            });
            r.dispatch_shortcut('Action1');
            assert(!flag);

            r.enabled = true;
            r.dispatch_shortcut('Action1');
            assert(flag);
        });
    });
});
