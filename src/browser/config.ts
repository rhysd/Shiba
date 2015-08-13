import {load as loadYAML} from 'js-yaml';
import {join} from 'path';
import {readFileSync} from 'fs';
import {getPath} from 'app';

export interface Config {
    linter: string;
    file_ext: {
        markdown: string[];
        html: string[];
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
}

export function load(): Config {
    if (this.user_config) {
        return this.user_config;
    }

    const config_dir = getPath('userData');
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
    };

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
        this.user_config = loadYAML(readFileSync(file, {encoding: 'utf8'}));
        mergeConfig(this.user_config, default_config);
    } catch(e) {
        console.log('No configuration file is found: ' + file);
        this.user_config = default_config;
    }

    this.user_config._config_dir_path = config_dir;

    return this.user_config;
}
