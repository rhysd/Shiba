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
        file_ext: ["md", "markdown", "mkd"]
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
