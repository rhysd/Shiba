import type { Root as Hast } from 'hast';
import * as log from './log';
import { MessageFromMain } from './ipc';
import { registerKeymaps } from './keymaps';
import { parseMarkdown, searchHast, PreviewContent } from './preview';

export function countSearchMatches(): number {
    return document.querySelectorAll('.search-text,.search-text-current').length;
}

export interface SearchState {
    text: string;
    index: number | null;
}

export interface State {
    search: SearchState | null;
    preview: PreviewContent | null;
}

export const INITIAL_STATE: State = {
    search: null,
    preview: null,
};

type Action =
    | {
          kind: 'open_search';
      }
    | {
          kind: 'close_search';
          content: PreviewContent;
      }
    | {
          kind: 'search_text';
          text: string;
          content: PreviewContent;
          index: number | null;
      }
    | {
          kind: 'preview_content';
          content: PreviewContent;
          searchText: string;
      };
export type Dispatch = React.Dispatch<Action>;

export function reducer(state: State, action: Action): State {
    log.debug('Dispatched new action', action.kind, action);
    switch (action.kind) {
        case 'open_search':
            if (state.search !== null) {
                return state; // When search is ongoing, do not update the state
            }
            return { ...state, search: { text: '', index: null } };
        case 'close_search':
            return { ...state, search: null, preview: action.content };
        case 'search_text':
            return {
                ...state,
                search: {
                    text: action.text,
                    index: action.index,
                },
                preview: action.content,
            };
        case 'preview_content':
            let search = null;
            if (state.search) {
                search = {
                    text: action.searchText,
                    index: null,
                };
            }
            return {
                ...state,
                preview: action.content,
                search,
            };
        default:
            throw new Error(`Unknown action: ${action}`);
    }
}

export function openSearch(): Action {
    return { kind: 'open_search' };
}
export async function closeSearch(hast: Hast): Promise<Action> {
    const react = await searchHast(hast, '', 0);
    const content = { react, hast };
    return { kind: 'close_search', content };
}
export async function searchText(hast: Hast, text: string, index: number | null): Promise<Action> {
    const react = await searchHast(hast, text, index);
    const content = { react, hast };
    return { kind: 'search_text', text, content, index };
}
export async function searchNext(index: number | null, hast: Hast, text: string): Promise<Action> {
    const count = countSearchMatches();
    let next = 0;
    if (index !== null && count > 0) {
        next = index + 1 >= count ? 0 : index + 1;
    }
    return searchText(hast, text, next);
}
export async function searchPrevious(index: number | null, hast: Hast, text: string): Promise<Action> {
    const count = countSearchMatches();
    let next = 0;
    if (index !== null && count > 0) {
        next = index > 0 ? index - 1 : count - 1;
    }
    return searchText(hast, text, next);
}
export async function previewContent(markdown: string, searchText: string): Promise<Action> {
    const content = await parseMarkdown(markdown, searchText);
    return { kind: 'preview_content', content, searchText };
}

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
                    this.dispatch(await searchNext(search.index, preview.hast, search.text));
                    break;
                }
                case 'search_previous': {
                    const { search, preview } = this.state;
                    if (search === null || preview === null) {
                        break;
                    }
                    this.dispatch(await searchPrevious(search.index, preview.hast, search.text));
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
