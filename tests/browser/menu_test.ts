import {BrowserWindow} from 'electron';
import * as M from '../../browser/menu';

context('menu', () => {
    describe('#build', () => {
        it('returns menu object', () => {
            if (on_travis) {
                let w = new BrowserWindow({show: false});
                const m = M.build(w);
                assert.ok(m);
            }
        });
    });
});
