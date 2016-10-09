import {ipcRenderer} from 'electron';

interface Ipc {
    on(c: ChannelFromMain, callback: Electron.IpcRendererEventListener): this;
    send(c: ChannelFromRenderer, ...args: any[]): void;
}
const ipc: Ipc = ipcRenderer;

function onConfig(config: AppConfig) {
    // TODO
    console.log('CONFIG:', config);
}

export function setupReceivers() {
    ipc.on('shiba:send-config', (_, conf) => onConfig(conf));
}

export const send = ipc.send;
