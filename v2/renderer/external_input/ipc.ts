import {ipcRenderer} from 'electron';
import log from '../log';
import Store from '../store';
import {ActionKind} from '../actions';

interface Ipc {
    on(c: ChannelFromMain, callback: Electron.IpcRendererEventListener): this;
}
const ipc: Ipc = ipcRenderer;

export function setupReceivers() {
    ipc.on('shiba:send-config', (_, config) => {
        log.debug('shiba:send-config --> ', config);
        Store.dispatch({
            type: ActionKind.SetConfig,
            config,
        });
    });

    ipc.on('shiba:file-update', (_: any, id: number, file: string, change: string) => {
        log.debug('shiba:file-update -->', id, file, change);
        // TODO
    });

    ipc.on('shiba:dog-ready', (_: any, id: number, watching: string) => {
        log.debug('shiba:dog-ready -->', id, watching);
        Store.dispatch({
            type: ActionKind.NewTab,
            config: {}, // TODO: Get the directory/file local configuration
            id,
            path: watching,
        });
        // Note:
        // We can't render preview at first time here because the path may be
        // directory. In the case, we can't determine which file should be rendered.
    });
}

