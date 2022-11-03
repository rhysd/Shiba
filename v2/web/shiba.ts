import * as log from './log';
import { sendMessage, MessageFromMain } from './ipc';
import { registerKeymaps } from './keymaps';
import { Markdown, ContentCallback } from './markdown';

export class Shiba {
    private init: boolean;
    private markdown: Markdown;

    constructor() {
        this.init = false;
        this.markdown = new Markdown();
    }

    registerPreviewCallback(callback: ContentCallback): void {
        this.markdown.registerCallback(callback);
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
                    await this.markdown.parse(msg.content);
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
