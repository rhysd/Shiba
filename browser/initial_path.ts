import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';

function resolveSpecifiedPath(specified: string): string {
    if (specified === '') {
        return '';
    }
    if (specified[0] === '~') {
        const home = os.homedir();
        if (home === '') {
            return '';
        }
        return resolveSpecifiedPath(home + specified.slice(1));
    }
    if (!fs.existsSync(specified)) {
        console.log('Path specified in default_watch_path in config.yml does not exist:', specified);
        return '';
    }
    return specified;
}

function defaultPath(user_config_path: string) {
    const specified = resolveSpecifiedPath(user_config_path);
    if (specified !== '') {
        return specified;
    }

    const cwd = process.cwd();
    if (process.platform === 'darwin' && cwd === '/') {
        const doc_dir = path.join(process.env.HOME, 'Documents');
        if (fs.existsSync(doc_dir)) {
            return doc_dir;
        } else {
            return path.join(process.resourcesPath, 'README.md');
        }
    }

    return cwd;
}

export = function initial_path(specified: string): string {
    // Note:
    // First argument is a path to Electron binary and second one is a path to Shiba app directory.
    // So argv.length <= 2 means no path was specified.
    const executable = process.argv[0];
    const directly_executed = executable.endsWith(process.platform === 'win32' ? 'Shiba.exe' : 'Shiba');
    if (process.argv.length <= (directly_executed ? 1 : 2)) {
        return defaultPath(specified);
    }

    const last_arg = process.argv[process.argv.length - 1];
    if (fs.existsSync(last_arg)) {
        return path.resolve(last_arg);
    } else {
        console.log(`Specified path '${last_arg}' not found. Ignored.`);
        return defaultPath(specified);
    }
};

