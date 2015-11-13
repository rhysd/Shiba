require('co-mocha')(require('mocha'));
const Nightmare = require('nightmare');
const assert = require('chai').assert;

describe('<lint-message>', () => {
    var nightmare;

    beforeEach(() => {
        nightmare = Nightmare({
            'web-preferences': {
                'node-integration': true
            }
        });
    });

    afterEach(function*() {
        yield nightmare.end();
    });

    it('exists in test html', function*() {
        const tag = yield nightmare
            .goto('file://' + __dirname + '/lint-message_test.html')
            .evaluate(() => document.querySelector('.test').tagName);
        assert.strictEqual(tag, 'LINT-MESSAGE');
    });

    it('renders header content', function*() {
        const attrs = yield nightmare
            .goto('file://' + __dirname + '/lint-message_test.html')
            .evaluate(() => {
                const node = document.querySelector('.test');
                return [node.getAttribute('header'), node.getAttribute('body')];
            });
        assert.strictEqual(attrs[0], 'This is header');
        assert.strictEqual(attrs[1], 'This is body');
    });
});
