import type { Root as Hast } from 'hast';
import * as log from './log';
import { parseMarkdown, searchHast, PreviewContent } from './preview';

export function countSearchMatches(): number {
    return document.querySelectorAll('.search-text,.search-text-current').length;
}

export interface SearchState {
    query: string;
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
          kind: 'search_query';
          query: string;
          content: PreviewContent;
          index: number | null;
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
            let search = null;
            if (state.search) {
                search = {
                    query: action.query,
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

// Action creators

export function openSearch(): Action {
    return { kind: 'open_search' };
}
export async function closeSearch(hast: Hast): Promise<Action> {
    const react = await searchHast(hast, '', 0);
    const content = { react, hast };
    return { kind: 'close_search', content };
}
export async function searchQuery(hast: Hast, query: string, index: number | null): Promise<Action> {
    const react = await searchHast(hast, query, index);
    const content = { react, hast };
    return { kind: 'search_query', query, content, index };
}
export async function searchNext(index: number | null, hast: Hast, query: string): Promise<Action> {
    const count = countSearchMatches();
    let next = 0;
    if (index !== null && count > 0) {
        next = index + 1 >= count ? 0 : index + 1;
    }
    return searchQuery(hast, query, next);
}
export async function searchPrevious(index: number | null, hast: Hast, query: string): Promise<Action> {
    const count = countSearchMatches();
    let next = 0;
    if (index !== null && count > 0) {
        next = index > 0 ? index - 1 : count - 1;
    }
    return searchQuery(hast, query, next);
}
export async function previewContent(markdown: string, query: string): Promise<Action> {
    const content = await parseMarkdown(markdown, query);
    return { kind: 'preview_content', content, query };
}
