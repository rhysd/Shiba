import Mousetrap from 'mousetrap';
import * as log from './log';
import { sendMessage, KeyMaps } from './ipc';

const KEYMAP_ACTIONS: { [action: string]: () => void } = Object.freeze({
    ScrollDown(): void {
        window.scrollBy(0, window.innerHeight / 2);
    },
    ScrollUp(): void {
        window.scrollBy(0, -window.innerHeight / 2);
    },
    ScrollLeft(): void {
        window.scrollBy(-window.innerWidth / 2, 0);
    },
    ScrollRight(): void {
        window.scrollBy(window.innerWidth / 2, 0);
    },
    ScrollPageDown(): void {
        window.scrollBy(0, window.innerHeight);
    },
    ScrollPageUp(): void {
        window.scrollBy(0, -window.innerHeight);
    },
    Forward(): void {
        sendMessage({ kind: 'forward' });
    },
    Back(): void {
        sendMessage({ kind: 'back' });
    },
    Reload(): void {
        sendMessage({ kind: 'reload' });
    },
    OpenFile(): void {
        sendMessage({ kind: 'file_dialog' });
    },
    OpenDir(): void {
        sendMessage({ kind: 'dir_dialog' });
    },
    ScrollTop(): void {
        window.scrollTo(0, 0);
    },
    ScrollBottom(): void {
        window.scrollTo(0, document.body.scrollHeight);
    },
});

export function registerKeymaps(keymaps: KeyMaps): void {
    for (const [keybind, action] of Object.entries(keymaps)) {
        const callback = KEYMAP_ACTIONS[action];
        if (callback) {
            Mousetrap.bind(keybind, e => {
                e.preventDefault();
                e.stopPropagation();
                log.debug('Triggered key shortcut:', action);
                callback();
            });
        } else {
            log.error('Unknown action:', action);
        }
    }
}
