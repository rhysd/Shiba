import { marked } from 'marked';
import hljs from 'highlight.js';
import Mousetrap from 'mousetrap';

interface Ipc {
    postMessage(m: string): void;
}

declare global {
    interface Window {
        ShibaApp: Shiba;
        ipc: Ipc;
    }
}

type MessageFromMain =
    | {
          kind: 'content';
          content: string;
      }
    | {
          kind: 'key_mappings';
          keymaps: { [keybind: string]: string };
      }
    | {
          kind: 'debug';
      };
type MessageToMain =
    | {
          kind: 'init';
      }
    | {
          kind: 'forward';
      }
    | {
          kind: 'back';
      }
    | {
          kind: 'reload';
      }
    | {
          kind: 'file_dialog';
      }
    | {
          kind: 'dir_dialog';
      };

function sendMessage(m: MessageToMain): void {
    window.ipc.postMessage(JSON.stringify(m));
}

let debug: (...args: unknown[]) => void = function nop() {};

const KEYMAP_ACTIONS: { [action: string]: () => void } = {
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
};

marked.setOptions({
    highlight: (code, lang) => {
        const language = hljs.getLanguage(lang) ? lang : 'plaintext';
        return hljs.highlight(code, { language }).value;
    },
    langPrefix: 'hljs language-',
    gfm: true,
});

class Shiba {
    receive(msg: MessageFromMain): void {
        debug('Received IPC message from main:', msg.kind, msg);
        switch (msg.kind) {
            case 'content':
                const elem = document.getElementById('preview');
                if (elem === null) {
                    console.error("'preview' element is not found");
                    return;
                }
                elem.innerHTML = marked.parse(msg.content);
                break;
            case 'key_mappings':
                for (const [keybind, action] of Object.entries(msg.keymaps)) {
                    const callback = KEYMAP_ACTIONS[action];
                    if (callback) {
                        Mousetrap.bind(keybind, e => {
                            e.preventDefault();
                            e.stopPropagation();
                            debug('Triggered key shortcut:', action);
                            callback();
                        });
                    } else {
                        console.error('Unknown action:', action);
                    }
                }
                document.getElementById('preview')?.focus();
                document.getElementById('preview')?.click();
                break;
            case 'debug':
                debug = console.debug;
                debug('Debug log is enabled');
                break;
            default:
                console.error('Unknown message:', msg);
                break;
        }
    }
}

window.ShibaApp = new Shiba();
sendMessage({ kind: 'init' });
