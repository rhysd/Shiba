import {load as loadYAML} from 'js-yaml';
import {join} from 'path';
import {readFileSync} from 'fs';
import {app} from 'electron';

export interface Config {
    linter: string;
    file_ext: {
        markdown: string[];
        html: string[];
        [n: string]: string[];
    }
    width: number | string;
    height: number | string;
    shortcuts: Object;
    voice: {
        enabled: boolean;
        source: string;
    }
    lint_options?: any;
    drawer: {
        responsive: boolean;
    }
    menu: {
        visible: boolean;
    }
    ignore_path_pattern: string;
    [name: string]: any;
}

export function load(): Config {
    if (this.user_config) {
        return this.user_config;
    }

    const config_dir = app.getPath('userData');
    const file = join(config_dir, 'config.yml');
    const default_config = {
        linter: "mdast-lint",
        file_ext: {
            markdown: ["md", "markdown", "mkd"],
            html: ["html"]
            // TODO: Add slim?
        },
        width: 800,
        height: 600,
        ignore_path_pattern: "[\\\\/]\\.",
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
        shortcuts: {
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
        }
    } as Config;

    function mergeConfig(c1: Config, c2: Config) {
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
        this.user_config = loadYAML(readFileSync(file, {encoding: 'utf8'})) as Config;
        mergeConfig(this.user_config, default_config);
    } catch(e) {
        console.log('No configuration file is found: ' + file);
        this.user_config = default_config;
    }

    this.user_config._config_dir_path = config_dir;

    return this.user_config;
}
