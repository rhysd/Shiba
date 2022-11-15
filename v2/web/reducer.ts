import * as log from './log';
import type { SearchMatcher } from './ipc';
import { searchNextIndex, searchPreviousIndex } from './search';

export interface State {
    searching: boolean;
    searchIndex: number | null;
    matcher: SearchMatcher;
}

export const INITIAL_STATE: State = {
    searching: false,
    searchIndex: null,
    matcher: 'SmartCase',
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
            if (state.matcher === action.matcher) {
                return state;
            }
            return { ...state, matcher: action.matcher };
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
