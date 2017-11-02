/// <reference path="lib.d.ts" />

import {load as loadYAML} from 'js-yaml';
import {join} from 'path';
import {readFile} from 'fs';
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
        css_path: '../../node_modules/github-markdown-css/github-markdown.css',
        code_theme: 'github',
    },
    path_watcher: {
        follow_symlinks: false,
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
    for (const k of Object.keys(c2)) {
        const v2 = c2[k];

        if (k in c1) {
            const v1 = c1[k];
            if (typeof(v1) === 'object' && typeof(v2) === 'object') {
                mergeConfig(v1, v2);
            }
            continue;
        }
        c1[k] = c2[k];
    }
    return c1;
}

function readConfigYAML(config_dir: string): Promise<string> {
    return new Promise((resolve, reject) => {
        const yml = join(config_dir, 'config.yml');
        readFile(yml, 'utf8', (yml_err, yml_content) => {
            if (!yml_err) {
                resolve(yml_content);
                return;
            }

            const yaml = join(config_dir, 'config.yaml');
            readFile(yaml, 'utf8', (yaml_err, yaml_content) => {
                if (!yaml_err) {
                    resolve(yaml_content);
                    return;
                }

                reject(new Error(`config.yml nor config.yaml was not found in '${config_dir}'`));
            });
        });
    });
}

let cache: Config | null = null;

export default function loadConfig(): Promise<Config> {
    if (cache !== null) {
        return Promise.resolve(cache);
    }

    const config_dir = app.getPath('userData');
    return readConfigYAML(config_dir).then(content => {
        cache = loadYAML(content) as Config;
        mergeConfig(cache, default_config);
        cache._config_dir_path = config_dir;
        return cache;
    }).catch(err => {
        console.log(err.message);
        cache = default_config;
        cache._config_dir_path = config_dir;
        return default_config;
    });
}
