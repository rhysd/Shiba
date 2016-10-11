import {ipcRenderer} from 'electron';
import log from '../log';

interface Ipc {
    on(c: ChannelFromMain, callback: Electron.IpcRendererEventListener): this;
}
const ipc: Ipc = ipcRenderer;

function onConfig(config: AppConfig) {
    // TODO
    log.debug('CONFIG:', config);
}

export function setupReceivers() {
    ipc.on('shiba:send-config', (_, conf) => onConfig(conf));
}

