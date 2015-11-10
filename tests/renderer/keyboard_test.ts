/// <reference path="./lib.d.ts" />
/// <reference path="../../renderer/keyboard.ts" />

context('Keyboard', () => {
    describe('Receiver', () => {
        describe('#constructor', () => {
            it('generates receiver object', () => {
                assert.ok(Keyboard.Receiver);
                assert.ok(new Keyboard.Receiver({}));
            });
        });
    });
});
