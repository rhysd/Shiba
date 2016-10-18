import * as path from 'path';
import * as fs from 'fs';
import {remote} from 'electron';
import * as recommended from 'remark-preset-lint-recommended';
import * as consistent from 'remark-preset-lint-consistent';
import log from '../log';

interface Presets {
    [name: string]: Remark.Preset;
}

type Rules = Object;

const AllGlobalPresets: Presets = {
    recommended,
    consistent,
};

let UserDefinedPresetCache = null as Presets;

export function loadUserInstalledPresets(): Promise<Presets> {
    return new Promise<Presets>(resolve => {
        if (UserDefinedPresetCache !== null) {
            return resolve(UserDefinedPresetCache);
        }

        const data_dir = remote.app.getPath('userData');
        const node_modules = path.join(data_dir, 'node_modules');
        UserDefinedPresetCache = {};

        fs.readdir(node_modules, (err, entries) => {
            if (err) {
                // User does not use node_modules
                log.debug('readdir failed because of node_modules not found. Skipped.', err);
                return resolve(UserDefinedPresetCache);
            }

            UserDefinedPresetCache = {};
            for (const e of entries) {
                if (e.startsWith('remark-preset-lint-')) {
                    const p = path.join(node_modules, e);
                    const name = e.slice('remark-preset-lint-'.length);
                    try {
                        const presets = require(p);
                        UserDefinedPresetCache[name] = presets;
                        AllGlobalPresets[name] = presets;
                    } catch (e) {
                        log.error('Failed to load user-installed preset module:', p, e);
                    }
                }
            }
            log.debug('Loaded user-installed presets:', UserDefinedPresetCache);
            resolve(UserDefinedPresetCache);
        });
    });
}

// There are some steps to load linter presets. Latter has higher priority on conflict.
//
// 1. Default preset: consistent
// 2. Specified presets in global user config `config.linter.remark_lint.presets`
// 3. Specified rules in global user config `config.linter.remark_lint.rules`
// 4. Specified presets in local user config `config.linter.remark_lint.presets`
// 5. Specified rules in local user config `config.linter.remark_lint.rules`

function loadRulesFromPresets(presets: Presets, config: RemarkLintConfig): Rules {

    const rules: Rules = {};
    if (config.presets) {
        for (const name of config.presets) {
            if (name in presets) {
                const preset = presets[name];
                Object.assign(rules, preset.plugins.lint);
            } else {
                log.error('Preset specified in config.yaml is not found:', name);
            }
        }
    }

    if (config.rules) {
        Object.assign(rules, config.rules);
    }

    return rules;
}

export function loadGlobalRulesSync(global_config: RemarkLintConfig): Rules {
    if (AllGlobalPresets === null) {
        throw new Error('Fatal: Global presets are not loaded yet.');
    }

    return loadRulesFromPresets(AllGlobalPresets, global_config);
}

export function loadGlobalRules(global_config: RemarkLintConfig): Promise<Rules> {
    return loadUserInstalledPresets()
        .then(() => loadGlobalRulesSync(global_config));
}

export function loadRulesSync(local_config: RemarkLintConfig, global_config: RemarkLintConfig): Rules {
    return Object.assign(
        loadGlobalRulesSync(global_config),
        loadRulesFromPresets(AllGlobalPresets, local_config)
    );
}

export function loadRules(local_config: RemarkLintConfig, global_config: RemarkLintConfig): Promise<Rules> {
    return loadUserInstalledPresets().then(() => loadRulesSync(local_config, global_config));
}
