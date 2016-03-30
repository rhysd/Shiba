/// <reference path="lib.d.ts" />
/// <reference path="../../renderer/emoji.ts" />

import {runInThisContext} from 'vm';
import {readFileSync} from 'fs';
import {join} from 'path';
import * as assert from 'power-assert';

const dependant_script = join(__dirname, '..', '..', '..', '..', 'build', 'src', 'renderer', 'emoji.js');
runInThisContext(readFileSync(dependant_script, 'utf8'));

describe('Emoji.Replacer', () => {
    describe('#replaceWithImages', () => {
        it('does nothing to text including no emoji', () => {
            const r = new Emoji.Replacer(__dirname);
            assert(r.replaceWithImages('') === '');
            assert(r.replaceWithImages('foo bar') === 'foo bar');
            assert(r.replaceWithImages(':bar') === ':bar');
            assert(r.replaceWithImages('foo :bar') === 'foo :bar');
            assert(r.replaceWithImages('foo :<b>foo</b>:') === 'foo :<b>foo</b>:');
        });

        it('does nothing to unknown emoji', () => {
            const r = new Emoji.Replacer(__dirname);
            assert(r.replaceWithImages(':boku_no_kangaeta_saikyouno_emoji:') === ':boku_no_kangaeta_saikyouno_emoji:');
            assert(
                r.replaceWithImages('Ah, I do not know :this_emoij:. Do not :replace_this_emoji:.')
                === 'Ah, I do not know :this_emoij:. Do not :replace_this_emoji:.'
            );
        });

        it('replaces emojis with <img> tag', () => {
            const r = new Emoji.Replacer(__dirname);
            assert(r.replaceWithImages(':dog:') === `<img src="${__dirname}/emoji/dog.png" title=":dog:" height="16px"></img>`);
            assert(
                r.replaceWithImages(':dog: walks with you.') ===
                `<img src="${__dirname}/emoji/dog.png" title=":dog:" height="16px"></img> walks with you.`
            );
            assert(
                r.replaceWithImages(':dog: walks with :cat:.') ===
                `<img src="${__dirname}/emoji/dog.png" title=":dog:" height="16px"></img> walks with <img src="${__dirname}/emoji/cat.png" title=":cat:" height="16px"></img>.`
            );
            assert(
                r.replaceWithImages(':dog: walks with :foo:.') ===
                `<img src="${__dirname}/emoji/dog.png" title=":dog:" height="16px"></img> walks with :foo:.`
            );
        });
    });
});
