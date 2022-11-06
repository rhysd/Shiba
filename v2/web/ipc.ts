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
    | 'SearchPrev';

export type KeyMaps = { [keybind: string]: KeyAction };
export type SearchMatcher = 'SmartCase' | 'CaseSensitive' | 'CaseInsensitive' | 'CaseSensitiveRegex';

export type MessageFromMain =
    | {
          kind: 'content';
          content: string;
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
        console.error('Could not send message to the main:', err);
    }
}
