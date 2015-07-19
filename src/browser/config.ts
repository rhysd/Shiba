import yaml = require('js-yaml');
import path = require('path');
import fs = require('fs');
import app = require('app');

export function load(): any/*TODO*/ {
    if (this.user_config) {
        return this.user_config;
    }

    const file = path.join(app.getPath('userData'), 'config.yml');
    const default_config = {
        linter: "mdast-lint",
        file_ext: ["md", "markdown", "mkd"],
        width: 800,
        height: 600,
        shortcuts: {
            'J':         'PageDown',
            'K':         'PageUp',
            'Down':      'PageDown',
            'Up':        'PageUp',
            'PageDown':  'PageDown',
            'PageUp':    'PageUp',
            'H':         'PageLeft',
            'L':         'PageRight',
            'Left':      'PageLeft',
            'Right':     'PageRight',
            'I':         'PageTop',
            'M':         'PageBottom',
            'Home':      'PageTop',
            'End':       'PageBottom',
            'Shift+J':   'PageBottom',
            'Control+P': 'ChangePath',
            'Control+L': 'Lint'
        }
    };

    function mergeConfig(c1, c2) {
        for (const k in c2) {
            const v2 = c2[k];

            if (k in c1) {
                let v1 = c1[k];
                if (typeof(v1) === "object" && typeof(v2) === "object") {
                    mergeConfig(v1, v2);
                }
                continue;
            }

            c1[k] = c2[k];
        }
    }

    try {
        this.user_config = yaml.load(fs.readFileSync(file, {encoding: 'utf8'}));
        mergeConfig(this.user_config, default_config);
    } catch(e) {
        console.log('No configuration file is found: ' + file);
        this.user_config = default_config;
    }

    return this.user_config;
}
