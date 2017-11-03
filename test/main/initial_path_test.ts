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
        process.argv = argv();
        assert.equal(initial_path(''), process.cwd());
    });

    it('returns document directory if started with Shiba.app in darwin', () => {
        if (process.platform === 'darwin') {
            process.argv = argv();
            process.chdir('/');
            assert(/Documents$/.test(initial_path('')));
        }
    });

    it('returns the specified path if it exists', () => {
        process.argv = argv('./README.md');
        assert.equal(initial_path(''), path.resolve('./README.md'));
    });

    it('returns default path if specified path does not exist', () => {
        process.argv = argv('/this/file/does/not/exist');
        assert.equal(initial_path(''), process.cwd());
    });

    it('returns the last argument of argv if multiple argument specified', () => {
        process.argv = argv('foo', 'bar', './README.md');
        assert.equal(initial_path(''), path.resolve('./README.md'));
    });

    it('considers the case where the Electron binary is directly executed', () => {
        let exe = '/path/to/Shiba';
        if (process.platform === 'win32') {
            exe = 'C:\\path\\to\\Shiba.exe';
        }

        process.argv = [exe];
        assert.equal(initial_path(''), process.cwd());

        process.argv = [exe, './README.md'];
        assert.equal(initial_path(''), path.resolve('./README.md'));
    });
});
