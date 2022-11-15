import Mousetrap from 'mousetrap';
import {
    type Dispatch,
    type State,
    INITIAL_STATE,
    openSearch,
    searchNext,
    searchPrevious,
    setSearchMatcher,
} from './reducer';
import { sendMessage, type MessageFromMain, type KeyAction } from './ipc';
import { PreviewContent } from './markdown';
import * as log from './log';

// Global action dispatcher to handle IPC messages from the main

export class GlobalDispatcher {
    public dispatch: Dispatch;
    public state: State;
    public content: PreviewContent;

    constructor() {
        this.dispatch = () => {
            // do nothing by default
        };
        this.state = INITIAL_STATE;
        this.content = new PreviewContent();
    }

    setDispatch(dispatch: Dispatch, state: State): void {
        this.dispatch = dispatch;
        this.state = state;
    }

    openSearch(): void {
        this.dispatch(openSearch());
    }

    searchNext(): void {
        const { searching, searchIndex } = this.state;
        if (searching) {
            this.dispatch(searchNext(searchIndex));
        }
    }

    searchPrev(): void {
        const { searching, searchIndex } = this.state;
        if (searching) {
            this.dispatch(searchPrevious(searchIndex));
        }
    }

    // Note: Passing message as JSON string and parse it with JSON.parse may be faster.
    // https://v8.dev/blog/cost-of-javascript-2019#json
    handleIpcMessage(msg: MessageFromMain): void {
        log.debug('Received IPC message from main:', msg.kind, msg);
        // This method must not throw exception since the main process call this method like `window.ShibaApp.receive(msg)`.
        try {
            switch (msg.kind) {
                case 'render_tree':
                    this.content.render(msg.tree);
                    break;
                case 'config':
                    for (const keybind of Object.keys(msg.keymaps)) {
                        const action = msg.keymaps[keybind];
                        Mousetrap.bind(keybind, e => {
                            e.preventDefault();
                            e.stopPropagation();
                            log.debug('Triggered key shortcut:', action, 'by', keybind);
                            try {
                                this.handleKeyAction(action);
                            } catch (err) {
                                log.error('Error while handling key action', action, err);
                            }
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
            case 'Quit':
                sendMessage({ kind: 'quit' });
                break;
            default:
                log.error('Unknown key action:', action);
                break;
        }
    }
}
