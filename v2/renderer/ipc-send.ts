import {ipcRenderer} from 'electron';

export const send = ipcRenderer.send as (c: ChannelFromRenderer, ...args: any[]) => void;
export const sendSync = ipcRenderer.sendSync as (c: ChannelFromRenderer, ...args: any[]) => void;

