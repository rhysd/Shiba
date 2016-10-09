import * as path from 'path';
import {readFileSync, writeFileSync} from 'fs';
import * as yaml from 'js-yaml';
import * as chokidar from 'chokidar';
import {app, BrowserWindow} from 'electron';
import log from './log';
import Ipc from './ipc';

export const DEFAULT_CONFIG = {
    linter: {
        remark_lint: {
            enabled: true,
            presets: ['lint-consistent'],
            rules: [],
        },
        proselint: {
            enabled: false,
            // TODO
        },
        textlint: {
            enabled: false,
            // TODO
        },
    },
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
    /* tslint:disable:object-literal-key-quotes */
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
    /* tslint:enable:object-literal-key-quotes */
} as AppConfig;

function loadConfigFromFile(dir: string): AppConfig | null {
    try {
        return yaml.safeLoad(readFileSync(path.join(dir, 'config.yaml'), {encoding: 'utf8'})) as AppConfig;
    } catch (_) {
        try {
            log.debug('YAML config file was not found. Falling back to JSON file', dir);
            return JSON.parse(readFileSync(path.join(dir, 'config.json'), {encoding: 'utf8'})) as AppConfig;
        } catch (_) {
            return null;
        }
    }
}

function loadOrCreateConfigFile(dir: string) {
    const config = loadConfigFromFile(dir);
    if (config !== null) {
        config._config_dir_path = dir;
        if (config.drawer) {
            log.warn("'drawer' option was removed. It'll be ignored.");
        }
        if (config.markdown) {
            log.warn("Deprecated configration 'markdown' will be converted to 'preview_customize.markdown'");
            config.preview_customize = {
                markdown: config.markdown,
            };
        }
        log.debug('Configuration was loaded from', dir, config);
        global.application_config = config;
        return config;
    } else {
        const file = path.join(dir, 'config.yaml');
        writeFileSync(file, yaml.safeDump(DEFAULT_CONFIG));
        log.info('New configuration file created:', file);
        DEFAULT_CONFIG._config_dir_path = dir;
        global.application_config = DEFAULT_CONFIG;
        return config;
    }
}

export default function loadAppConfig() {
    return new Promise<AppConfig>(resolve => {
        if (global.application_config) {
            return resolve(global.application_config);
        }
        const dir = app.getPath('userData');
        resolve(loadOrCreateConfigFile(dir));

        Ipc.onReceive('shiba:request-config', function (this: Electron.IpcMainEvent) {
            this.sender.send('shiba:send-config', global.application_config);
        });

        chokidar.watch(path.join(dir, 'config.(yaml|json)'), {
            ignoreInitial: true,
            persistent: true,
        }).on('add', () => {
            const config = loadOrCreateConfigFile(dir);
            log.debug('Configuration was update. Will send a new configuration to renderer process', config);
            const wins = BrowserWindow.getAllWindows();
            if (wins.length === 0) {
                log.warn('No browser window was found! Updating configuration will be skipped');
                return;
            }
            wins[0].webContents.send('shiba:send-config', config);
        });
    });
}
