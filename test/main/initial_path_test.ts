/// <reference path="lib.d.ts" />

import initial_path = require('../../browser/initial_path');
import * as path from 'path';
import * as assert from 'power-assert';

function argv(...a: string[]) {
    return ['path/to/Electron', 'path/to/Shiba', ...a];
}

describe('#initial_path()', () => {
    let argv_save: string[];
    let cwd_save: string;

    beforeEach(() => {
        argv_save = process.argv;
        cwd_save = process.cwd();
    });

    afterEach(() => {
        process.argv = argv_save;
        process.chdir(cwd_save);
    });

    it('returns cwd when no argument is specified in general platform', () => {
        process.argv = [];
        assert(initial_path() === process.cwd());
    });

    it('returns document directory if started with Shiba.app in darwin', () => {
        if (process.platform === 'darwin') {
            process.argv = [];
            process.chdir('/');
            assert(/Documents$/.test(initial_path()));
        }
    });

    it('returns the specified path if it exists', () => {
        process.argv = argv('./README.md');
        assert(initial_path() === path.resolve('./README.md'));
    });

    it('returns default path if specified path does not exist', () => {
        process.argv = argv('/this/file/does/not/exist');
        assert(initial_path() === process.cwd());
    });

    it('returns the last argument of argv if multiple argument specified', () => {
        process.argv = argv('foo', 'bar', './README.md');
        assert(initial_path() === path.resolve('./README.md'));
    });
});
