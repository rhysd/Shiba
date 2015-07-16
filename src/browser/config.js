'use strict';

var yaml = require('js-yaml');
var path = require('path');
var fs = require('fs');
var app = require('app');

// Note:
// Add Config class to allow to deal with default config and user
// customization

module.exports.load = function(){
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
            'H':         'PageLeft',
            'L':         'PageRight',
            'Shift+J':   'PageBottom',
            'Shift+K':   'PageTop',
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
        this.user_config = yaml.load(fs.readFileSync(file));
        mergeConfig(this.user_config, default_config);
    } catch(e) {
        console.log('No configuration file is found: ' + file);
        this.user_config = default_config;
    }

    return this.user_config;
};
