import * as log from './log';
import type { SearchMatcher } from './ipc';

export function findSearchMatchElems(): NodeListOf<HTMLElement> {
    return document.querySelectorAll('.search-text,.search-text-current');
}

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
            return { ...state, searching: false };
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
    const elems = findSearchMatchElems();
    for (const elem of elems) {
        elem.className = '';
    }
    return { kind: 'close_search' };
}

export function searchIndex(index: number | null): Action {
    return { kind: 'search_index', index };
}

export function searchNext(index: number | null): Action {
    const elems = findSearchMatchElems();

    let next;
    if (elems.length === 0) {
        next = null;
    } else if (index !== null) {
        next = index + 1 >= elems.length ? 0 : index + 1;
    } else {
        // Find the nearest next item against current scroll position
        const y = window.scrollY;
        for (const [i, e] of elems.entries()) {
            if (e.offsetTop >= y) {
                next = i;
                break;
            }
        }
        next ??= 0;
    }

    if (index !== next) {
        if (index !== null) {
            elems[index].className = 'search-text';
        }
        if (next !== null) {
            elems[next].className = 'search-text-current';
        }
    }

    return searchIndex(next);
}

export function searchPrevious(index: number | null): Action {
    const elems = findSearchMatchElems();

    let next;
    if (elems.length === 0) {
        next = null;
    } else if (index !== null) {
        next = index > 0 ? index - 1 : elems.length - 1;
    } else {
        // Find the nearest previous item against current scroll position
        const height = window.innerHeight || document.documentElement.clientHeight;
        const y = window.scrollY + height;
        for (const [i, e] of elems.entries()) {
            const bottom = e.offsetTop + e.clientHeight;
            if (bottom >= y) {
                next = i - 1;
                break;
            }
        }
        next = next !== undefined && next >= 0 ? next : elems.length - 1;
    }

    if (index !== next) {
        if (index !== null) {
            elems[index].className = 'search-text';
        }
        if (next !== null) {
            elems[next].className = 'search-text-current';
        }
    }

    return searchIndex(next);
}

export function setSearchMatcher(matcher: SearchMatcher): Action {
    return { kind: 'search_matcher', matcher };
}
