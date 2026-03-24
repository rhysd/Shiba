import {
    type Dispatch,
    type State,
    type Action,
    INITIAL_STATE,
    initialize,
    notifyAlwaysOnTop,
    notifyReload,
    notifyZoom,
    openHelp,
    openHistory,
    openOutline,
    openSearch,
    previewContent,
    searchNext,
    setPath,
    searchPrevious,
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
    public readonly markdown: ReactMarkdownRenderer;
    private fragment: string;

    constructor() {
        this.dispatch = (action: Action) => {
            log.error('Action is ignored by dispatcher because dispatch function is not set yet:', action);
        };
        this.state = INITIAL_STATE;
        this.keymap = new KeyMapping();
        this.markdown = new ReactMarkdownRenderer();
        this.fragment = '';
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

    async handleIpcMessage(msg: MessageFromMain): Promise<void> {
        log.debug('Received IPC message from main:', msg.kind, msg);
        // This method must not throw exception since the main process call this method like `window.postShibaMessageFromMain(msg)`.
        try {
            switch (msg.kind) {
                case 'render_tree': {
                    const tree = await this.markdown.render(msg.tree, this.fragment);
                    this.fragment = '';
                    this.dispatch(previewContent(tree));
                    break;
                }
                case 'path': {
                    this.dispatch(setPath(msg.path));
                    break;
                }
                case 'config':
                    this.keymap.register(msg.keymaps, this);
                    this.dispatch(
                        initialize(
                            {
                                titleBar: !msg.window.title,
                                vibrant: msg.window.vibrancy,
                                hideScrollBar: !msg.window.scrollBar,
                                borderTop: msg.window.borderTop,
                                homeDir: msg.home,
                            },
                            msg.search.matcher,
                        ),
                    );
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
                    this.dispatch(openHistory(msg.paths));
                    break;
                case 'welcome':
                    this.dispatch(welcome());
                    break;
                case 'help':
                    this.dispatch(openHelp());
                    break;
                case 'zoomed':
                    this.dispatch(notifyZoom(msg.percent));
                    break;
                case 'reload':
                    this.dispatch(notifyReload());
                    break;
                case 'always_on_top':
                    this.dispatch(notifyAlwaysOnTop(msg.pinned));
                    break;
                case 'next_fragment':
                    this.fragment = msg.hash;
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
