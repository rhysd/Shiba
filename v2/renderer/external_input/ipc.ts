import {ipcRenderer} from 'electron';
import log from '../log';
import Store from '../store';
import * as A from '../actions/type';
import {updatePreview, startPreview} from '../actions/preview';
import {loadUserInstalledPresets} from '../markdown/preset_loader';

interface Ipc {
    on(c: ChannelFromMain, callback: Electron.IpcRendererEventListener): this;
}
const ipc: Ipc = ipcRenderer;

export function setupReceivers() {
    ipc.on('shiba:send-config', (_: any, config: AppConfig) => {
        log.debug('shiba:send-config --> ', config);
        Store.dispatch({
            type: A.Kind.SetConfig,
            config,
        });
        // Note: Optimization: Load remark lint presets in advance
        loadUserInstalledPresets();
    });

    ipc.on('shiba:file-update', (_: any, id: number, file: string, change: string) => {
        log.debug('shiba:file-update -->', id, file, change);
        updatePreview(id, file)
            .then(a => Store.dispatch(a))
            .catch(() => { /* Do nothing */ });
    });

    ipc.on('shiba:dog-ready', (_: any, id: number, watching: string) => {
        log.debug('shiba:dog-ready -->', id, watching);
        startPreview(id, watching)
            .then(a => Store.dispatch(a))
            .catch(() => { /* Do nothing */ });
    });
}
