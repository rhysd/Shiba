import * as log from './log';
import type { SearchMatcher } from './ipc';
import { searchNextIndex, searchPreviousIndex } from './search';

export interface State {
    searching: boolean;
    searchIndex: number | null;
    matcher: SearchMatcher;
    previewing: boolean;
}

export const INITIAL_STATE: State = {
    searching: false,
    searchIndex: null,
    matcher: 'SmartCase',
    previewing: true,
};

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
      };
export type Dispatch = React.Dispatch<Action>;

export function reducer(state: State, action: Action): State {
    log.debug('Dispatched new action', action.kind, action);
    switch (action.kind) {
        case 'open_search':
            if (state.searching) {
                return state;
            }
            return { ...state, searching: true, searchIndex: null };
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
