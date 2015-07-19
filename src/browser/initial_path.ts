import fs = require('fs');
import path = require('path');

export = function() {
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
    // First argument is a path to Shiba app
    if (process.argv.length < 2) {
        return defaultPath();
    }

    let last_arg = process.argv[process.argv.length-1];
    if (fs.existsSync(last_arg)) {
        return path.resolve(last_arg);
    } else {
        return defaultPath();
    }
};

