import * as fs from 'fs';
import * as path from 'path';
import {ipcRenderer} from 'electron';
import {ReactElement} from 'react';
import log from '../log';
import Store from '../store';
import {ActionKind} from '../actions';
import {createProcessor} from '../markdown/processor';
import {loadUserInstalledPresets} from '../markdown/preset_loader';

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
        // Note: Optimization: Load remark lint presets in advance
        loadUserInstalledPresets();
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
        const global_config = Store.getState().tabs.transformConfig;

        // Note:
        // Should we use lstat to stat symlinks?
        fs.stat(watching, (err, stats) => {
            if (err) {
                log.error('Error on statting path:', watching, 'Error:', err);
                return;
            }
            if (stats.isFile()) {
                createProcessor(path.dirname(watching), global_config).then(processor => {
                    log.debug('Processor created for', watching, processor);
                    processor.processFile(watching).then(v => {
                        log.debug('First compilation succeeded for directory:', watching, 'Result:', v);
                        Store.dispatch({
                            type: ActionKind.NewTab,
                            preview: {
                                id,
                                processor,
                                watchingPath: watching,
                                contents: v.contents,
                            },
                        });
                    });
                });
            } else if (stats.isDirectory()) {
                createProcessor(watching, global_config).then(processor => {
                    log.debug('Processor created for', watching, processor);
                    Store.dispatch({
                        type: ActionKind.NewTab,
                        preview: {
                            id,
                            processor,
                            watchingPath: watching,
                            contents: null,
                        },
                    });
                });
            } else {
                log.error('Watching path is not a file nor a directory:', watching, 'Stats:', stats);
            }
        });
    });
}
