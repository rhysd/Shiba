import * as yaml from 'js-yaml';
import * as path from 'path';
import {readFileSync, writeFileSync} from 'fs';
import {app} from 'electron';

export const DEFAULT_CONFIG = {
    linter: 'remark-lint',
    lint_options: null,
    file_ext: {
        markdown: ['md', 'markdown', 'mkd'],
    },
    width: 920,
    height: 800,
    restore_window_state: true,
    ignore_path_pattern: '[\\\\/]\\.',
    voice: null,
    menu: {
        visible: true,
    },
    hide_title_bar: false,
    hide_menu_bar: true,
    preview_customize: null,
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
        's':        'Search',
        'o':        'Outline',
    },
} as AppConfig;

export default function loadAppConfig() {
    return new Promise<AppConfig>(resolve => {
        const dir = app.getPath('userData');
        const file = path.join(dir, 'config.yaml');
        try {
            const config = yaml.safeLoad(readFileSync(file, {encoding: 'utf8'})) as AppConfig;
            config._config_dir_path = dir;
            if (config.drawer) {
                console.error("'drawer' option was removed. It'll be ignored.");
            }
            if (config.markdown) {
                config.preview_customize = {
                    markdown: config.markdown,
                };
            }
            resolve(config);
        } catch (_) {
            writeFileSync(file, yaml.safeDump(DEFAULT_CONFIG));
            console.log('New configuration file created: ' + file);
            DEFAULT_CONFIG._config_dir_path = dir;
            resolve(DEFAULT_CONFIG);
        }
    });
}
