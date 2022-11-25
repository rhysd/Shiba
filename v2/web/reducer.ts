import * as log from './log';
import type { SearchMatcher, Theme as WindowTheme } from './ipc';
import { searchNextIndex, searchPreviousIndex } from './search';

type Theme = 'light' | 'dark';

export type NotificationContent =
    | {
          kind: 'zoom';
          percent: number;
      }
    | {
          kind: 'reload';
      };

export interface State {
    searching: boolean;
    searchIndex: number | null;
    matcher: SearchMatcher;
    previewing: boolean;
    outline: boolean;
    theme: Theme;
    history: boolean;
    files: string[];
    help: boolean;
    notifying: boolean;
    notification: NotificationContent;
}

export const INITIAL_STATE: State = {
    searching: false,
    searchIndex: null,
    matcher: 'SmartCase',
    previewing: true,
    outline: false,
    theme: 'light',
    history: false,
    files: [],
    help: false,
    notifying: false,
    notification: { kind: 'reload' },
};

const MAX_HISTORIES = 50;

type Action =
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
          kind: 'previewing';
          previewing: boolean;
      }
    | {
          kind: 'outline';
          open: boolean;
      }
    | {
          kind: 'theme';
          theme: Theme;
      }
    | {
          kind: 'history';
          open: boolean;
      }
    | {
          kind: 'new_file';
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
      };
export type Dispatch = React.Dispatch<Action>;

export function reducer(state: State, action: Action): State {
    log.debug('Dispatched new action', action.kind, action);
    switch (action.kind) {
        case 'new_file': {
            const index = state.files.indexOf(action.path);
            if (index >= 0) {
                const files = state.files.slice(0, index);
                for (let i = index + 1; i < state.files.length; i++) {
                    files.push(state.files[i]);
                }
                files.push(state.files[index]);
                return { ...state, files };
            } else if (state.files.length >= MAX_HISTORIES) {
                state.files.push(action.path);
                return {
                    ...state,
                    files: state.files.slice(1),
                };
            } else {
                return {
                    ...state,
                    files: [...state.files, action.path],
                };
            }
        }
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
        case 'previewing':
            return { ...state, previewing: action.previewing };
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
        case 'theme':
            return { ...state, theme: action.theme };
        case 'recent_files':
            return { ...state, files: action.paths };
        default:
            throw new Error(`Unknown action: ${action}`);
    }
}

// Action creators

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

export function setPreviewing(previewing: boolean): Action {
    return { kind: 'previewing', previewing };
}

export function openOutline(): Action {
    return { kind: 'outline', open: true };
}

export function closeOutline(): Action {
    return { kind: 'outline', open: false };
}

export function setTheme(theme: WindowTheme): Action {
    return {
        kind: 'theme',
        theme: theme === 'Dark' ? 'dark' : 'light',
    };
}

export function openHistory(): Action {
    return { kind: 'history', open: true };
}

export function closeHistory(): Action {
    return { kind: 'history', open: false };
}

export function newFile(path: string): Action {
    return { kind: 'new_file', path };
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

export function setRecentFiles(paths: string[]): Action {
    return { kind: 'recent_files', paths };
}
