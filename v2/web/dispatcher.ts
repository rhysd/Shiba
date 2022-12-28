import {
    type Dispatch,
    type State,
    INITIAL_STATE,
    previewContent,
    openSearch,
    searchNext,
    searchPrevious,
    setSearchMatcher,
    openOutline,
    setTheme,
    newFile,
    openHistory,
    openHelp,
    notifyZoom,
    notifyReload,
    notifyAlwaysOnTop,
    setRecentFiles,
    welcome,
} from './reducer';
import type { MessageFromMain } from './ipc';
import { ReactMarkdownRenderer } from './markdown';
import { KeyMapping } from './keymaps';
import * as log from './log';

// Global action dispatcher to handle IPC messages from the main and key shortcuts

export class GlobalDispatcher {
    public dispatch: Dispatch; // This prop will be updated by `App` component
    public state: State; // This prop will be updated by `App` component
    public readonly keymap: KeyMapping;

    constructor() {
        this.dispatch = () => {
            // do nothing by default
        };
        this.state = INITIAL_STATE;
        this.keymap = new KeyMapping();
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

    handleIpcMessage(msg: MessageFromMain): void {
        log.debug('Received IPC message from main:', msg.kind, msg);
        // This method must not throw exception since the main process call this method like `window.postShibaMessageFromMain(msg)`.
        try {
            switch (msg.kind) {
                case 'render_tree': {
                    const renderer = new ReactMarkdownRenderer();
                    const tree = renderer.renderMarkdown(msg.tree);
                    this.dispatch(previewContent(tree));
                    break;
                }
                case 'new_file':
                    this.dispatch(newFile(msg.path));
                    break;
                case 'config':
                    this.keymap.register(msg.keymaps, this);
                    this.dispatch(setTheme(msg.theme));
                    this.dispatch(setSearchMatcher(msg.search.matcher));
                    this.dispatch(setRecentFiles(msg.recent));
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
                case 'outline':
                    this.dispatch(openOutline());
                    break;
                case 'history':
                    this.dispatch(openHistory());
                    break;
                case 'welcome':
                    this.dispatch(welcome());
                    break;
                case 'help':
                    this.dispatch(openHelp());
                    break;
                case 'zoom':
                    this.dispatch(notifyZoom(msg.percent));
                    break;
                case 'reload':
                    this.dispatch(notifyReload());
                    break;
                case 'always_on_top':
                    this.dispatch(notifyAlwaysOnTop(msg.pinned));
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
