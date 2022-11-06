import type { Root as Hast } from 'hast';
import * as log from './log';
import type { SearchMatcher } from './ipc';
import { parseMarkdown, searchHast, PreviewContent } from './preview';

export function findSearchMatchElems(): NodeListOf<HTMLElement> {
    return document.querySelectorAll('.search-text,.search-text-current');
}

export interface SearchState {
    query: string;
    index: number | null;
}

export interface State {
    search: SearchState | null;
    preview: PreviewContent | null;
    matcher: SearchMatcher;
}

export const INITIAL_STATE: State = {
    search: null,
    preview: null,
    matcher: 'SmartCase',
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
          kind: 'search_query';
          query: string;
          content: PreviewContent;
          index: number | null;
      }
    | {
          kind: 'search_matcher';
          matcher: SearchMatcher;
      }
    | {
          kind: 'preview_content';
          content: PreviewContent;
          query: string;
      };
export type Dispatch = React.Dispatch<Action>;

export function reducer(state: State, action: Action): State {
    log.debug('Dispatched new action', action.kind, action);
    switch (action.kind) {
        case 'open_search':
            if (state.search !== null) {
                return state; // When search is ongoing, do not update the state
            }
            return { ...state, search: { query: '', index: null } };
        case 'close_search':
            return { ...state, search: null, preview: action.content };
        case 'search_query':
            return {
                ...state,
                search: {
                    query: action.query,
                    index: action.index,
                },
                preview: action.content,
            };
        case 'preview_content':
            return {
                ...state,
                preview: action.content,
                search: state.search
                    ? {
                          query: action.query,
                          index: null,
                      }
                    : null,
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

// TODO: Do not re-render the content by searchHast on searchNext/searchPrevious/closeSearch.
// These actions can be implemented by directly modifying DOM tree (modifying/removing class names). They are much faster
// than re-rendering content especially when the content is quite big.

export function openSearch(): Action {
    return { kind: 'open_search' };
}
export async function closeSearch(hast: Hast): Promise<Action> {
    const react = await searchHast(hast, '', null, 'SmartCase');
    const content = { react, hast };
    return { kind: 'close_search', content };
}
export async function searchQuery(
    hast: Hast,
    query: string,
    index: number | null,
    matcher: SearchMatcher,
): Promise<Action> {
    const react = await searchHast(hast, query, index, matcher);
    const content = { react, hast };
    return { kind: 'search_query', query, content, index };
}
export function searchNext(index: number | null, hast: Hast, query: string, matcher: SearchMatcher): Promise<Action> {
    const elems = findSearchMatchElems();
    let next;
    if (elems.length === 0) {
        next = 0;
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
    return searchQuery(hast, query, next, matcher);
}
export function searchPrevious(
    index: number | null,
    hast: Hast,
    query: string,
    matcher: SearchMatcher,
): Promise<Action> {
    const elems = findSearchMatchElems();
    let next;
    if (elems.length === 0) {
        next = 0;
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
    return searchQuery(hast, query, next, matcher);
}
export function setSearchMatcher(matcher: SearchMatcher): Action {
    return { kind: 'search_matcher', matcher };
}
export async function previewContent(markdown: string, query: string, matcher: SearchMatcher): Promise<Action> {
    const content = await parseMarkdown(markdown, query, matcher);
    return { kind: 'preview_content', content, query };
}
