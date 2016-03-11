import {replaceAll} from '../../browser/emoji';

context('Emoji', () =>
    describe('#replaceAll', () => {
        it('does nothing to text including no emoji', () => {
            assert.strictEqual(replaceAll(''), '');
            assert.strictEqual(replaceAll('foo bar'), 'foo bar');
            assert.strictEqual(replaceAll(':bar'), ':bar');
            assert.strictEqual(replaceAll('foo :bar'), 'foo :bar');
        });

        it('does nothing to unknown emoji', () => {
            assert.strictEqual(replaceAll(':boku_no_kangaeta_saikyouno_emoji:'), ':boku_no_kangaeta_saikyouno_emoji:');
            assert.strictEqual(
                replaceAll('Ah, I do not know :this_emoij:. Do not :replace_this_emoji:.'),
                'Ah, I do not know :this_emoij:. Do not :replace_this_emoji:.'
            );
        });

        it('replaces emojis with <img> tag', () => {
            assert.match(replaceAll(':dog:'), /^<img [^>]+><\/img>$/);
            assert.match(replaceAll(':dog: walks with you.'), /^<img [^>]+><\/img> walks with you\.$/);
            assert.match(replaceAll(':dog: walks with :cat:.'), /^<img [^>]+><\/img> walks with <img [^>]+><\/img>\.$/);
            assert.match(replaceAll(':dog: walks with :foo:.'), /^<img [^>]+><\/img> walks with :foo:\.$/);
        });
    })
);
