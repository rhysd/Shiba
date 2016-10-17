import * as fs from 'fs';
import {ipcRenderer} from 'electron';
import {ReactElement} from 'react';
import log from '../log';
import Store from '../store';
import {ActionKind} from '../actions';
import MarkdownProcessor from '../markdown/processor';

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

        const tabs = Store.getState().tabs;
        const preview = tabs.previews.get(id);
        if (!preview) {
            log.error('Invalid ID was updated: id', id, 'previews:', tabs.previews);
            return;
        }

        const action = {
            type: ActionKind.UpdatePreview,
            id,
            contents: null as ReactElement<any>,
        };

        if (preview.id !== tabs.currentId) {
            log.debug('File updated but not a current id preview. Updated:', preview.id, 'Curent:', tabs.currentId);
            Store.dispatch(action);
            return;
        }

        preview.processor.processFile(file).then(v => {
            log.debug('Converted new preview for file:', file);
            action.contents = v.contents;
            Store.dispatch(action);
        });
        // TODO: Error handling?
    });

    ipc.on('shiba:dog-ready', (_: any, id: number, watching: string) => {
        log.debug('shiba:dog-ready -->', id, watching);
        const default_config = Store.getState().tabs.transformConfig;
        // TODO: Get the directory/file local configuration
        const config = Object.assign({}, default_config || {});
        const processor = new MarkdownProcessor(config);
        const action = {
            type: ActionKind.NewTab,
            preview: {
                id,
                processor,
                watchingPath: watching,
                contents: null as ReactElement<any>,
            },
        };

        // Note:
        // Should we use lstat to stat symlinks?
        fs.stat(watching, (err, stats) => {
            if (err) {
                log.error('Error on statting path:', watching, 'Error:', err);
                return;
            }
            if (stats.isFile()) {
                processor.processFile(watching).then(v => {
                    action.preview.contents = v.contents;
                    Store.dispatch(action);
                });
            } else if (stats.isDirectory()) {
                Store.dispatch(action);
            } else {
                log.error('Watching path is not a file nor a directory:', watching, 'Stats:', stats);
            }
        });
    });
}

