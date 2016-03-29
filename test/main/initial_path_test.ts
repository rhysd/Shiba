/// <reference path="lib.d.ts" />

import initial_path = require('../../browser/initial_path');
import * as path from 'path';
import * as assert from 'power-assert';

describe('#initial_path()', () => {
    const argv = process.argv;
    const cwd = process.cwd();
    afterEach(() => {
        process.argv = argv;
        process.chdir(cwd);
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
        process.argv = ['dummy', './README.md'];
        assert(initial_path() === path.resolve('./README.md'));
    });

    it('returns default path if specified path does not exist', () => {
        process.argv = ['dummy', '/this/file/does/not/exist'];
        assert(initial_path() === process.cwd());
    });

    it('returns the last argument of argv if multiple argument specified', () => {
        process.argv = ['dummy', 'foo', 'bar', './README.md'];
        assert(initial_path() === path.resolve('./README.md'));
    });
});
