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
    | 'Quit';

export type KeyMaps = { [keybind: string]: KeyAction };
export type SearchMatcher = 'SmartCase' | 'CaseSensitive' | 'CaseInsensitive' | 'CaseSensitiveRegex';

export type ParseTreeTableAlign = 'left' | 'center' | 'right' | null;
export interface ParseTreeFootNoteDef {
    t: 'fn-def';
    name?: string;
    id: number;
    c: ParseTreeElem[];
}
// Note: 't' is abbreviation of 'tag', 'c' is abbreviation of 'children' for saving spaces
export type ParseTreeElem =
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
          c: ParseTreeElem[];
      }
    | {
          t: 'h';
          level: number;
          id?: string;
          c: ParseTreeElem[];
      }
    | {
          t: 'table';
          align: ParseTreeTableAlign[];
          c: ParseTreeElem[];
      }
    | {
          t: 'thead';
          c: ParseTreeElem[];
      }
    | {
          t: 'tbody';
          c: ParseTreeElem[];
      }
    | {
          t: 'tr';
          c: ParseTreeElem[];
      }
    | {
          t: 'th';
          c: ParseTreeElem[];
      }
    | {
          t: 'td';
          c: ParseTreeElem[];
      }
    | {
          t: 'blockquote';
          c: ParseTreeElem[];
      }
    | {
          t: 'pre';
          c: ParseTreeElem[];
      }
    | {
          t: 'code';
          lang?: string;
          c: ParseTreeElem[];
      }
    | {
          t: 'ol';
          start?: number;
          c: ParseTreeElem[];
      }
    | {
          t: 'ul';
          c: ParseTreeElem[];
      }
    | {
          t: 'li';
          c: ParseTreeElem[];
      }
    | {
          t: 'em';
          c: ParseTreeElem[];
      }
    | {
          t: 'strong';
          c: ParseTreeElem[];
      }
    | {
          t: 'del';
          c: ParseTreeElem[];
      }
    | {
          t: 'a';
          href: string;
          title?: string;
          c: ParseTreeElem[];
      }
    | {
          t: 'img';
          src: string;
          title?: string;
          c: ParseTreeElem[]; // Note: Children are alt text
      }
    | ParseTreeFootNoteDef
    | {
          t: 'modified'; // Special token to indicate the last modified position
      };

export type MessageFromMain =
    | {
          kind: 'parse_tree';
          tree: ParseTreeElem[];
      }
    | {
          kind: 'config';
          keymaps: KeyMaps;
          search: {
              matcher: SearchMatcher;
          };
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
