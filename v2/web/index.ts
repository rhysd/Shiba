import { mount } from './components';
import * as log from './log';
import { sendMessage, MessageFromMain } from './ipc';
import { registerKeymaps } from './keymaps';
import { parseMarkdown } from './markdown';

declare global {
    interface Window {
        ShibaApp: Shiba;
    }
}

type ContentCallback = (elem: React.ReactNode) => void;

class Shiba {
    private onMarkdownContent: ContentCallback;
    private init: boolean;

    constructor() {
        this.onMarkdownContent = () => {};
        this.init = false;
    }

    registerContentCallback(callback: ContentCallback): void {
        this.onMarkdownContent = callback;
        if (!this.init) {
            sendMessage({ kind: 'init' });
            this.init = true;
            log.debug('Notify initialization finished to the main');
        }
        log.debug('Registered new content callback');
    }

    async receive(msg: MessageFromMain): Promise<void> {
        log.debug('Received IPC message from main:', msg.kind, msg);

        // This method must not throw exception since the main process call this method like `window.ShibaApp.receive(msg)`.
        try {
            switch (msg.kind) {
                case 'content':
                    this.onMarkdownContent(await parseMarkdown(msg.content));
                    break;
                case 'key_mappings':
                    registerKeymaps(msg.keymaps);
                    break;
                case 'debug':
                    log.enableDebug();
                    log.debug('Debug log is enabled');
                    break;
                default:
                    log.error('Unknown message:', msg);
                    break;
            }
        } catch (err) {
            log.error('Error while handling received IPC message', err, msg);
        }
    }
}

window.ShibaApp = new Shiba(); // The main process sends events via `window.ShibaApp` global variable
mount(document.getElementById('shiba-root')!);
