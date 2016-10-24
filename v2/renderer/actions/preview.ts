import * as fs from 'fs';
import * as path from 'path';
import {ReactElement} from 'react';
import Store from '../store';
import * as A from '../actions/type';
import {createProcessor} from '../markdown/processor';
import log from '../log';

type AsyncAction = Promise<A.Type>;

export function updatePreview(id: number, file: string): AsyncAction {
    const tabs = Store.getState().tabs;
    const preview = tabs.previews.get(id);
    if (!preview) {
        const reason = `Invalid ID was updated: id ${id}, previews: ${tabs.previews}`;
        log.error(reason);
        return Promise.reject(reason);
    }

    const action = {
        type: A.Kind.UpdatePreview,
        id,
        contents: null as ReactElement<any>,
    };

    if (preview.id !== tabs.currentId) {
        log.debug('File updated but not a current id preview. Updated:', preview.id, 'Curent:', tabs.currentId);
        return Promise.resolve(action);
    }

    return preview.processor.processFile(file).then(v => {
        log.debug('Converted new preview for file:', file);
        action.contents = v.contents;
        return action;
    });
    // TODO: Error handling?
}

export function startPreview(id: number, watching: string): AsyncAction {
    const global_config = Store.getState().tabs.transformConfig;
    return new Promise<A.Type>((resolve, reject) => {
        // Note:
        // Should we use lstat to stat symlinks?
        fs.stat(watching, (err, stats) => {
            if (err) {
                const reason = `Error on statting path: ${watching}, Error: ${err.message}`;
                log.error(reason);
                return reject(reason);
            }
            if (stats.isFile()) {
                createProcessor(path.dirname(watching), global_config).then(processor => {
                    log.debug('Processor created for', watching, processor);
                    processor.processFile(watching).then(v => {
                        log.debug('First compilation succeeded for directory:', watching, 'Result:', v);
                        resolve({
                            type: A.Kind.NewTab,
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
                    resolve({
                        type: A.Kind.NewTab,
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
                reject('Watching path is not a file nor a directory:' + watching);
            }
        });
    });
}
