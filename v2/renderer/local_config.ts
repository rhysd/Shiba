import * as fs from 'fs';
import * as path from 'path';
import log from './log';

function resolveLocalConfigImpl(root: string, dir: string): Promise<string | null> {
    return new Promise<string | null>(resolve => {
        fs.readdir(dir, (err, children) => {
            if (err) {
                log.error('Directory not found to search:', dir);
                return resolve(null);
            }

            let git_root = false;
            for (const child of children) {
                if (child === '.shibarc.json') {
                    return resolve(path.join(dir, child));
                } else if (child === '.git') {
                    git_root = true;
                }
            }

            const next = path.dirname(dir);
            if (git_root || next === root) {
                // Do not search outside a Git repository
                return resolve(null);
            }

            return resolve(resolveLocalConfigImpl(root, next));
        });
    });
}

export function resolveLocalConfig(dir: string): Promise<string | null> {
    return resolveLocalConfigImpl(path.parse(dir).root, dir);
}

export function resolveLocalConfigSync(dir: string): string | null {
    const root = path.parse(dir).root;
    while (root !== dir) {
        let git_root = false;
        try {
            const children = fs.readdirSync(dir);
            for (const child of children) {
                if (child === '.shibarc.json') {
                    return path.join(dir, child);
                } else if (child === '.git') {
                    git_root = true;
                }
            }
        } catch (e) {
            log.error('Directory not found to search:', dir);
            return null;
        }

        if (git_root) {
            // Do not search outside a Git repository
            return null;
        }

        dir = path.dirname(dir);
    }
    return null;
}

function parseLocalConfig(file: string): Promise<AppConfig | null> {
    return new Promise<AppConfig>(resolve => {
        fs.readFile(file, 'utf8', (err, data) => {
            if (err) {
                log.error('Error on loading local configration from', file, 'Error:', err);
                return resolve(null);
            }
            return JSON.parse(data);
        });
    });
}

export default function loadLocalConfigFor(dir: string): Promise<AppConfig | null> {
    return resolveLocalConfig(dir).then(file => {
        if (file === null) {
            log.debug('Local config was not found for', dir);
            return null;
        }
        log.debug('Local config file was found', file);
        return parseLocalConfig(file);
    });
}
