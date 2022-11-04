import type { Root as Hast } from 'hast';
import * as log from './log';
import { MessageFromMain } from './ipc';
import { registerKeymaps } from './keymaps';
import { parseMarkdown, searchHast, PreviewContent } from './preview';

export interface State {
    search: boolean;
    searchText: string;
    preview: PreviewContent | null;
}

export const INITIAL_STATE: State = {
    search: false,
    searchText: '',
    preview: null,
};

export type Action =
    | {
          kind: 'open_search';
      }
    | {
          kind: 'close_search';
      }
    | {
          kind: 'search_text';
          text: string;
          content: PreviewContent;
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
            return { ...state, search: true, searchText: '' };
        case 'close_search':
            return { ...state, search: false, searchText: '' };
        case 'search_text':
            return {
                ...state,
                search: true,
                searchText: action.text,
                preview: action.content,
            };
        case 'preview_content':
            return { ...state, preview: action.content, searchText: action.searchText };
        default:
            throw new Error(`Unknown action: ${action}`);
    }
}

export function openSearch(): Action {
    return { kind: 'open_search' };
}
export function closeSearch(): Action {
    return { kind: 'close_search' };
}
export async function searchText(hast: Hast, text: string): Promise<Action> {
    const react = await searchHast(hast, text);
    const content = { react, hast };
    return { kind: 'search_text', text, content };
}
export async function previewContent(markdown: string, searchText: string): Promise<Action> {
    const content = await parseMarkdown(markdown, searchText);
    return { kind: 'preview_content', content, searchText };
}

export class Dispatcher {
    public dispatch: Dispatch;

    constructor() {
        this.dispatch = () => {};
    }

    setDispatch(dispatch: Dispatch): void {
        this.dispatch = dispatch;
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
