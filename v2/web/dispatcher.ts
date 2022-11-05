import { Dispatch, State, INITIAL_STATE, previewContent, openSearch, searchNext, searchPrevious } from './reducer';
import { registerKeymaps } from './keymaps';
import type { MessageFromMain } from './ipc';
import * as log from './log';

// Global action dispatcher to handle IPC messages from the main

export class Dispatcher {
    public dispatch: Dispatch;
    public state: State;

    constructor() {
        this.dispatch = () => {};
        this.state = INITIAL_STATE;
    }

    setDispatch(dispatch: Dispatch, state: State): void {
        this.dispatch = dispatch;
        this.state = state;
    }

    async dispatchIpcMessage(msg: MessageFromMain): Promise<void> {
        log.debug('Received IPC message from main:', msg.kind, msg);
        // This method must not throw exception since the main process call this method like `window.ShibaApp.receive(msg)`.
        try {
            switch (msg.kind) {
                case 'content':
                    this.dispatch(await previewContent(msg.content, ''));
                    break;
                case 'key_mappings':
                    registerKeymaps(msg.keymaps);
                    break;
                case 'search':
                    this.dispatch(openSearch());
                    break;
                case 'search_next': {
                    const { search, preview } = this.state;
                    if (search === null || preview === null) {
                        break;
                    }
                    this.dispatch(await searchNext(search.index, preview.hast, search.query));
                    break;
                }
                case 'search_previous': {
                    const { search, preview } = this.state;
                    if (search === null || preview === null) {
                        break;
                    }
                    this.dispatch(await searchPrevious(search.index, preview.hast, search.query));
                    break;
                }
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
