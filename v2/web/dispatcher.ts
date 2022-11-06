import Mousetrap from 'mousetrap';
import {
    Dispatch,
    State,
    INITIAL_STATE,
    previewContent,
    openSearch,
    searchNext,
    searchPrevious,
    closeSearch,
    setSearchMatcher,
} from './reducer';
import { sendMessage, MessageFromMain, KeyAction } from './ipc';
import * as log from './log';

// Global action dispatcher to handle IPC messages from the main

export class Dispatcher {
    public dispatch: Dispatch;
    public state: State;

    constructor() {
        this.dispatch = () => {
            // do nothing by default
        };
        this.state = INITIAL_STATE;
    }

    setDispatch(dispatch: Dispatch, state: State): void {
        this.dispatch = dispatch;
        this.state = state;
    }

    openSearch(): void {
        this.dispatch(openSearch());
    }

    async closeSearch(): Promise<void> {
        const { search, preview } = this.state;
        if (search !== null && preview !== null) {
            this.dispatch(await closeSearch(preview.hast));
        }
    }

    searchNext(): void {
        const { search } = this.state;
        if (search !== null) {
            this.dispatch(searchNext(search.index));
        }
    }

    searchPrev(): void {
        const { search } = this.state;
        if (search !== null) {
            this.dispatch(searchPrevious(search.index));
        }
    }

    async dispatchIpcMessage(msg: MessageFromMain): Promise<void> {
        log.debug('Received IPC message from main:', msg.kind, msg);
        // This method must not throw exception since the main process call this method like `window.ShibaApp.receive(msg)`.
        try {
            switch (msg.kind) {
                case 'content': {
                    const query = this.state.search?.query ?? '';
                    this.dispatch(await previewContent(msg.content, query, this.state.matcher));
                    break;
                }
                case 'config':
                    for (const keybind of Object.keys(msg.keymaps)) {
                        const action = msg.keymaps[keybind];
                        Mousetrap.bind(keybind, e => {
                            e.preventDefault();
                            e.stopPropagation();
                            log.debug('Triggered key shortcut:', action, 'by', keybind);
                            this.handleKeyAction(action);
                        });
                    }
                    this.dispatch(setSearchMatcher(msg.search.matcher));
                    break;
                case 'search':
                    this.openSearch();
                    break;
                case 'search_next':
                    this.searchNext();
                    break;
                case 'search_previous':
                    this.searchPrev();
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

    handleKeyAction(action: KeyAction): void {
        try {
            switch (action) {
                case 'ScrollDown':
                    window.scrollBy(0, window.innerHeight / 2);
                    break;
                case 'ScrollUp':
                    window.scrollBy(0, -window.innerHeight / 2);
                    break;
                case 'ScrollLeft':
                    window.scrollBy(-window.innerWidth / 2, 0);
                    break;
                case 'ScrollRight':
                    window.scrollBy(window.innerWidth / 2, 0);
                    break;
                case 'ScrollPageDown':
                    window.scrollBy(0, window.innerHeight);
                    break;
                case 'ScrollPageUp':
                    window.scrollBy(0, -window.innerHeight);
                    break;
                case 'Forward':
                    sendMessage({ kind: 'forward' });
                    break;
                case 'Back':
                    sendMessage({ kind: 'back' });
                    break;
                case 'Reload':
                    sendMessage({ kind: 'reload' });
                    break;
                case 'OpenFile':
                    sendMessage({ kind: 'file_dialog' });
                    break;
                case 'OpenDir':
                    sendMessage({ kind: 'dir_dialog' });
                    break;
                case 'ScrollTop':
                    window.scrollTo(0, 0);
                    break;
                case 'ScrollBottom':
                    window.scrollTo(0, document.body.scrollHeight);
                    break;
                case 'Search':
                    this.openSearch();
                    break;
                case 'SearchNext':
                    this.searchNext();
                    break;
                case 'SearchPrev':
                    this.searchPrev();
                    break;
                default:
                    log.error('Unknown key action:', action);
                    break;
            }
        } catch (err) {
            log.error('Could not handle key action', action, err);
        }
    }
}
