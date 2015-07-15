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
            'Control+D': 'ChangePath',
            'Control+L': 'Lint'
        }
    };

    try {
        this.user_config = yaml.load(fs.readFileSync(file));
        for (const k in default_config) {
            if (k in this.user_config) {
                continue;
            }
            this.user_config[k] = default_config[k];
        }
    } catch(e) {
        console.log('No configuration file is found: ' + file);
        this.user_config = default_config;
    }

    return this.user_config;
};
