interface Ipc {
    postMessage(m: string): void;
}

declare global {
    interface Window {
        ipc: Ipc;
    }
}

export type KeyMaps = { [keybind: string]: string };

export type MessageFromMain =
    | {
          kind: 'content';
          content: string;
      }
    | {
          kind: 'key_mappings';
          keymaps: KeyMaps;
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
      };

export function sendMessage(m: MessageToMain): void {
    window.ipc.postMessage(JSON.stringify(m));
}
