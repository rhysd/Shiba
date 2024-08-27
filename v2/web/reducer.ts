import * as log from './log';
import type { SearchMatcher } from './ipc';
import { searchNextIndex, searchPreviousIndex } from './search';
import type { MarkdownReactTree } from './markdown';

export type Theme = 'light' | 'dark';

export type NotificationContent =
    | {
          kind: 'zoom';
          percent: number;
      }
    | {
          kind: 'reload';
      }
    | {
          kind: 'alwaysOnTop';
          pinned: boolean;
      };

export interface Heading {
    level: number;
    text: string;
    elem: HTMLHeadingElement;
    current?: boolean;
}

export interface Config {
    titleBar: boolean;
    vibrant: boolean;
    hideScrollBar: boolean;
    borderTop: boolean;
    homeDir: string | null;
}

export interface State {
    previewTree: MarkdownReactTree;
    searching: boolean;
    searchIndex: number | null;
    matcher: SearchMatcher;
    outline: boolean;
    config: Config;
    history: boolean;
    files: string[];
    help: boolean;
    notifying: boolean;
    notification: NotificationContent;
    welcome: boolean;
    headings: Heading[];
    currentPath: string | null;
}

export const INITIAL_CONFIG: Config = {
    titleBar: true,
    vibrant: false,
    hideScrollBar: false,
    borderTop: false,
    homeDir: null,
};

export const INITIAL_STATE: State = {
    previewTree: {
        root: null,
        lastModified: null,
        matchCount: 0,
    },
    searching: false,
    searchIndex: null,
    matcher: 'SmartCase',
    outline: false,
    config: INITIAL_CONFIG,
    history: false,
    files: [],
    help: false,
    notifying: false,
    notification: { kind: 'reload' },
    welcome: false,
    headings: [],
    currentPath: null,
};

const MAX_HISTORIES = 50;

type Action =
    | {
          kind: 'preview_content';
          tree: MarkdownReactTree;
      }
    | {
          kind: 'open_search';
      }
    | {
          kind: 'close_search';
      }
    | {
          kind: 'search_index';
          index: number | null;
      }
    | {
          kind: 'search_index';
          index: number | null;
      }
    | {
          kind: 'search_matcher';
          matcher: SearchMatcher;
      }
    | {
          kind: 'outline';
          open: boolean;
      }
    | {
          kind: 'history';
          open: boolean;
      }
    | {
          kind: 'new_path';
          path: string;
      }
    | {
          kind: 'help';
          open: boolean;
      }
    | {
          kind: 'notification';
          notification: NotificationContent | null;
      }
    | {
          kind: 'recent_files';
          paths: string[];
      }
    | {
          kind: 'init';
          config: Config;
      }
    | {
          kind: 'headings';
          headings: Heading[];
      }
    | {
          kind: 'welcome';
      };
export type Dispatch = React.Dispatch<Action>;

export function reducer(state: State, action: Action): State {
    log.debug('Dispatched new action', action.kind, action);
    switch (action.kind) {
        case 'preview_content':
            return { ...state, previewTree: action.tree, welcome: false };
        case 'new_path': {
            const currentPath = action.path;
            const index = state.files.indexOf(currentPath);
            if (index >= 0) {
                const files = state.files.slice(0, index);
                for (let i = index + 1; i < state.files.length; i++) {
                    files.push(state.files[i]);
                }
                files.push(state.files[index]);
                return { ...state, files, currentPath };
            } else if (state.files.length >= MAX_HISTORIES) {
                state.files.push(currentPath);
                return {
                    ...state,
                    files: state.files.slice(1),
                    currentPath,
                };
            } else {
                return {
                    ...state,
                    files: [...state.files, currentPath],
                    currentPath,
                };
            }
        }
        case 'headings':
            return { ...state, headings: action.headings };
        case 'open_search':
            if (state.searching) {
                return state;
            }
            return { ...state, searching: true, searchIndex: null, outline: false, history: false, help: false };
        case 'close_search':
            return { ...state, searching: false, searchIndex: null };
        case 'search_index':
            if (!state.searching) {
                return state;
            }
            return {
                ...state,
                searchIndex: action.index,
            };
        case 'search_matcher':
            return { ...state, matcher: action.matcher };
        case 'outline':
            return { ...state, outline: action.open, searching: false, history: false, help: false };
        case 'history':
            return { ...state, history: action.open, searching: false, outline: false, help: false };
        case 'help':
            return { ...state, help: action.open, searching: false, outline: false, history: false };
        case 'notification':
            if (action.notification === null) {
                return { ...state, notifying: false };
            } else {
                return { ...state, notifying: true, notification: action.notification };
            }
        case 'init':
            return { ...state, config: action.config };
        case 'recent_files':
            return { ...state, files: action.paths };
        case 'welcome':
            return { ...state, welcome: true };
        default:
            throw new Error(`Unknown action: ${JSON.stringify(action)}`);
    }
}

// Action creators

export function previewContent(tree: MarkdownReactTree): Action {
    return { kind: 'preview_content', tree };
}

export function openSearch(): Action {
    return { kind: 'open_search' };
}

export function closeSearch(): Action {
    return { kind: 'close_search' };
}

export function searchIndex(index: number | null): Action {
    return { kind: 'search_index', index };
}

export function searchNext(index: number | null): Action {
    return searchIndex(searchNextIndex(index));
}

export function searchPrevious(index: number | null): Action {
    return searchIndex(searchPreviousIndex(index));
}

export function setSearchMatcher(matcher: SearchMatcher): Action {
    return { kind: 'search_matcher', matcher };
}

export function openOutline(): Action {
    return { kind: 'outline', open: true };
}

export function closeOutline(): Action {
    return { kind: 'outline', open: false };
}

export function openHistory(): Action {
    return { kind: 'history', open: true };
}

export function closeHistory(): Action {
    return { kind: 'history', open: false };
}

export function pathChanged(path: string): Action {
    return { kind: 'new_path', path };
}

export function openHelp(): Action {
    return { kind: 'help', open: true };
}

export function closeHelp(): Action {
    return { kind: 'help', open: false };
}

export function notifyZoom(percent: number): Action {
    return { kind: 'notification', notification: { kind: 'zoom', percent } };
}

export function dismissNotification(): Action {
    return { kind: 'notification', notification: null };
}

export function notifyReload(): Action {
    return { kind: 'notification', notification: { kind: 'reload' } };
}

export function notifyAlwaysOnTop(pinned: boolean): Action {
    return { kind: 'notification', notification: { kind: 'alwaysOnTop', pinned } };
}

export function setRecentFiles(paths: string[]): Action {
    return { kind: 'recent_files', paths };
}

export function welcome(): Action {
    return { kind: 'welcome' };
}

export function updateHeadings(headings: Heading[]): Action {
    return { kind: 'headings', headings };
}

export function initConfig(config: Config): Action {
    return {
        kind: 'init',
        config,
    };
}
