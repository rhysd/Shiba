/// <reference path="lib.d.ts" />
/// <reference path="../../renderer/emoji.ts" />

import {runInThisContext} from 'vm';
import {readFileSync} from 'fs';
import {join} from 'path';
import * as assert from 'power-assert';

const dependant_script = join(__dirname, '..', '..', '..', '..', 'build', 'src', 'renderer', 'emoji.js');
runInThisContext(readFileSync(dependant_script, 'utf8'));

describe('Emoji.Replacer', () => {
    describe('#replaceOne', () => {
        it('does nothing to text including no emoji', () => {
            const r = new Emoji.Replacer(__dirname);
            assert(r.replaceOne('') === '');
            assert(r.replaceOne('foo') === 'foo');
            assert(r.replaceOne(':bar') === ':bar');
            assert(r.replaceOne('foo :bar') === 'foo :bar');
            assert(r.replaceOne('foo :<b>foo</b>:') === 'foo :<b>foo</b>:');
        });

        it('does nothing to unknown emoji', () => {
            const r = new Emoji.Replacer(__dirname);
            assert(r.replaceOne('boku_no_kangaeta_saikyouno_emoji') === 'boku_no_kangaeta_saikyouno_emoji');
        });

        it('replaces emojis with <img> tag', () => {
            const r = new Emoji.Replacer(__dirname);
            assert(r.replaceOne('dog') === `<img src="${__dirname}/emoji/dog.png" title=":dog:" height="16px" alt="dog"></img>`);
        });
    });
});
