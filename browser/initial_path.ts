import * as fs from 'fs';
import * as path from 'path';

export = function(): string {
    function defaultPath() {
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

    // Note:
    // First argument is a path to Electron binary and second one is a path to Shiba app directory.
    // So argv.length <= 2 means no path was specified.
    if (process.argv.length <= 2) {
        return defaultPath();
    }

    const last_arg = process.argv[process.argv.length - 1];
    if (fs.existsSync(last_arg)) {
        return path.resolve(last_arg);
    } else {
        console.log(`Specified path '${last_arg}' not found. Ignored.`);
        return defaultPath();
    }
};

