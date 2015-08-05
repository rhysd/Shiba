import initial_path = require('../../src/browser/initial_path');
import * as path from 'path';

context('InitialPath', () =>
    describe('#initialPath()', () => {
        const argv = process.argv;
        afterEach(() => process.argv = argv);

        it('returns cwd when no argument is specified in general platform', () => {
            process.argv = [];
            if (process.platform !== 'darwin') {
                assert.strictEqual(initial_path(), process.cwd());
            }
        });

        it('returns specific path when no argument is specified in darwin', () => {
            if (process.platform === 'darwin') {
                process.argv = [];
                if (process.cwd() === '/') {
                    assert.match(initial_path(), /Documents$/);
                } else {
                    assert.strictEqual(initial_path(), path.join(process.resourcesPath, 'README.md'));
                }
            }
        });

        it('returns the specified path if it exists', () => {
            process.argv = ['dummy', './README.md'];
            assert.strictEqual(initial_path(), path.resolve('./README.md'));
        });

        it('returns default path path if specified path does not exist', () => {
            process.argv = ['dummy', '/this/file/does/not/exist'];
            assert.strictEqual(initial_path(), process.cwd());
        });

        it('returns the last argument of argv', () => {
            process.argv = ['dummy', 'foo', 'bar', './README.md'];
            assert.strictEqual(initial_path(), path.resolve('./README.md'));
        });
    })
);
