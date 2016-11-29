/// <reference path="lib.d.ts" />

import {load as loadYAML} from 'js-yaml';
import {join} from 'path';
import {readFileSync} from 'fs';
import {app} from 'electron';

export const default_config = {
    linter: 'remark-lint',
    file_ext: {
        markdown: ['md', 'markdown', 'mkd'],
        html: ['html'],
        // TODO: Add slim?
    },
    width: 920,
    height: 800,
    ignore_path_pattern: '[\\\\/]\\.',
    voice: {
        enabled: false,
        source: '../voices/bow.mp3',
    },
    drawer: {
        responsive: true,
    },
    menu: {
        visible: true,
    },
    hide_title_bar: false,
    hide_menu_bar: true,
    markdown: {
        font_size: '',
        css_path: '../../bower_components/github-markdown-css/github-markdown.css',
        code_theme: 'github',
    },
    restore_window_state: true,
    shortcuts: {
        /* tslint:disable:object-literal-key-quotes */
        'j':        'PageDown',
        'k':        'PageUp',
        'down':     'PageDown',
        'up':       'PageUp',
        'pagedown': 'PageDown',
        'pageup':   'PageUp',
        'h':        'PageLeft',
        'l':        'PageRight',
        'left':     'PageLeft',
        'right':    'PageRight',
        'i':        'PageTop',
        'm':        'PageBottom',
        'home':     'PageTop',
        'end':      'PageBottom',
        'ctrl+p':   'ChangePath',
        'ctrl+l':   'Lint',
        'r':        'Reload',
        's':        'Search',
        'o':        'Outline',
        /* tslint:enable:object-literal-key-quotes */
    },
} as Config;

function mergeConfig(c1: Config, c2: Config) {
    for (const k in c2) {
        const v2 = c2[k];

        if (k in c1) {
            let v1 = c1[k];
            if (typeof(v1) === 'object' && typeof(v2) === 'object') {
                mergeConfig(v1, v2);
            }
            continue;
        }
        c1[k] = c2[k];
    }
    return c1;
}

export default function loadConfig(): Promise<Config> {
    if (this.user_config) {
        return Promise.resolve(this.user_config);
    }

    return new Promise<Config>((resolve, reject) => {
        const config_dir = app.getPath('userData');
        const file = join(config_dir, 'config.yml');
        try {
            this.user_config = loadYAML(readFileSync(file, {encoding: 'utf8'})) as Config;
            mergeConfig(this.user_config, default_config);
        } catch (e) {
            console.log('No configuration file was found: ' + file);
            this.user_config = default_config;
        }

        this.user_config._config_dir_path = config_dir;

        resolve(this.user_config);
    });
}
