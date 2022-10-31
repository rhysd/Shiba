import hljs from 'highlight.js';
import Mousetrap from 'mousetrap';
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkFrontmatter from 'remark-frontmatter';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import rehypeHighlight from 'rehype-highlight';
import rehypeStringify from 'rehype-stringify';

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

const remark = unified()
    .use(remarkFrontmatter)
    .use(remarkGfm)
    .use(remarkParse)
    .use(remarkRehype)
    .use(rehypeHighlight, { plainText: ['txt', 'text'] })
    .use(rehypeStringify);

class Shiba {
    async receive(msg: MessageFromMain): Promise<void> {
        debug('Received IPC message from main:', msg.kind, msg);
        switch (msg.kind) {
            case 'content':
                const elem = document.getElementById('preview');
                if (elem === null) {
                    console.error("'preview' element is not found");
                    return;
                }

                const file = await remark.process(msg.content);
                elem.innerHTML = String(file);

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
