interface Ipc {
    postMessage(m: string): void;
}

declare global {
    interface Window {
        ipc: Ipc;
    }
}

export type KeyAction =
    | 'ScrollDown'
    | 'ScrollUp'
    | 'ScrollLeft'
    | 'ScrollRight'
    | 'ScrollPageDown'
    | 'ScrollPageUp'
    | 'Forward'
    | 'Back'
    | 'Reload'
    | 'OpenFile'
    | 'OpenDir'
    | 'ScrollTop'
    | 'ScrollBottom'
    | 'Search'
    | 'SearchNext'
    | 'SearchPrev'
    | 'ScrollNextSection'
    | 'ScrollPrevSection'
    | 'Outline'
    | 'History'
    | 'Help'
    | 'ZoomIn'
    | 'ZoomOut'
    | 'Quit';

export type KeyMaps = { [keybind: string]: KeyAction };
export type SearchMatcher = 'SmartCase' | 'CaseSensitive' | 'CaseInsensitive' | 'CaseSensitiveRegex';
export type Theme = 'Dark' | 'Light';

export type RenderTreeTableAlign = 'left' | 'center' | 'right' | null;
export interface RenderTreeFootNoteDef {
    t: 'fn-def';
    name?: string;
    id: number;
    c: RenderTreeElem[];
}
export interface RenderTreeMath {
    t: 'math';
    inline: boolean;
    expr: string;
}
// Note: 't' is abbreviation of 'tag', 'c' is abbreviation of 'children' for saving spaces
export type RenderTreeElem =
    | string // Text node
    | {
          t: 'html';
          raw: string;
      }
    | {
          t: 'br';
      }
    | {
          t: 'hr';
      }
    | {
          t: 'fn-ref';
          id: number;
      }
    | {
          t: 'checkbox';
          checked: boolean;
      }
    | {
          t: 'p';
          c: RenderTreeElem[];
      }
    | {
          t: 'h';
          level: number;
          id?: string;
          c: RenderTreeElem[];
      }
    | {
          t: 'table';
          align: RenderTreeTableAlign[];
          c: RenderTreeElem[];
      }
    | {
          t: 'thead';
          c: RenderTreeElem[];
      }
    | {
          t: 'tbody';
          c: RenderTreeElem[];
      }
    | {
          t: 'tr';
          c: RenderTreeElem[];
      }
    | {
          t: 'th';
          c: RenderTreeElem[];
      }
    | {
          t: 'td';
          c: RenderTreeElem[];
      }
    | {
          t: 'blockquote';
          c: RenderTreeElem[];
      }
    | {
          t: 'pre';
          c: RenderTreeElem[];
      }
    | {
          t: 'code';
          lang?: string;
          c: RenderTreeElem[];
      }
    | {
          t: 'ol';
          start?: number;
          c: RenderTreeElem[];
      }
    | {
          t: 'ul';
          c: RenderTreeElem[];
      }
    | {
          t: 'li';
          c: RenderTreeElem[];
      }
    | {
          t: 'em';
          c: RenderTreeElem[];
      }
    | {
          t: 'strong';
          c: RenderTreeElem[];
      }
    | {
          t: 'del';
          c: RenderTreeElem[];
      }
    | {
          t: 'a';
          href: string;
          title?: string;
          auto?: boolean; // Autolink
          c: RenderTreeElem[];
      }
    | {
          t: 'img';
          src: string;
          title?: string;
          c: RenderTreeElem[]; // Note: Children are alt text
      }
    | RenderTreeFootNoteDef
    | {
          t: 'emoji';
          name: string;
      }
    | RenderTreeMath
    | {
          t: 'modified'; // Special token to indicate the last modified position
      }
    | {
          t: 'match'; // Text search match tokens after match-start
          c: RenderTreeElem[];
      }
    | {
          t: 'match-current'; // Current text search match tokens after match-current-start
          c: RenderTreeElem[];
      }
    | {
          t: 'match-start'; // First text search match token
          c: RenderTreeElem[];
      }
    | {
          t: 'match-current-start'; // First current text search match token
          c: RenderTreeElem[];
      };

export type MessageFromMain =
    | {
          kind: 'render_tree';
          tree: RenderTreeElem[];
      }
    | {
          kind: 'new_file';
          path: string;
      }
    | {
          kind: 'config';
          keymaps: KeyMaps;
          search: {
              matcher: SearchMatcher;
          };
          theme: Theme;
          recent: string[];
      }
    | {
          kind: 'search';
      }
    | {
          kind: 'search_next';
      }
    | {
          kind: 'search_previous';
      }
    | {
          kind: 'outline';
      }
    | {
          kind: 'welcome';
      }
    | {
          kind: 'history';
      }
    | {
          kind: 'help';
      }
    | {
          kind: 'zoom';
          percent: number;
      }
    | {
          kind: 'reload';
      }
    | {
          kind: 'debug';
      };
export type MessageToMain =
    | {
          kind: 'init';
      }
    | {
          kind: 'quit';
      }
    | {
          kind: 'forward';
      }
    | {
          kind: 'back';
      }
    | {
          kind: 'reload';
      }
    | {
          kind: 'file_dialog';
      }
    | {
          kind: 'dir_dialog';
      }
    | {
          kind: 'search';
          query: string;
          index: number | null;
          matcher: SearchMatcher;
      }
    | {
          kind: 'open_file';
          path: string;
      }
    | {
          kind: 'zoom';
          zoom: 'In' | 'Out';
      }
    | {
          kind: 'error';
          message: string;
      };

export function sendMessage(m: MessageToMain): void {
    try {
        window.ipc.postMessage(JSON.stringify(m));
    } catch (err) {
        // Do not raise an error on sending IPC messages. Otherwise the renderer tries to send error message
        // to the main and it causes the same error again.
        console.error('Could not send message to the main:', err); // eslint-disable-line no-console
    }
}
