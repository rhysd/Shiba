import * as path from 'path';
import assert = require('assert');
import {Application} from 'spectron';

// XXX: Get Electron binary path
const electron: string = (require as any)('electron');

describe('Shiba', function () {
    this.timeout(10000);

    before(function () {
        this.app = new Application({
            path: electron,
            args: [path.join(__dirname, '..', '..')],
            env: {
                NODE_ENV: 'production',
            },
        });
        return this.app.start().then(() => this.app.client.pause(3000)); // Wait application starting
    });

    after(function () {
        if (this.app.isRunning()) {
            return this.app.stop();
        }
    });

    it('can start without an error', function () {
        return this.app.client.getWindowCount().then((c: number) => assert.equal(c, 1))
            .then(() => this.app.browserWindow.isVisible()).then((b: boolean) => assert.ok(b))
            .then(() => this.app.webContents.getURL()).then((u: string) => assert.ok(u))
            .then(() => this.app.client.execute(() => {
                const e = document.getElementById('main-drawer') as any;
                if (!e) {
                    throw new Error('main drawer not found!');
                }
            }))
            .then(() => this.app.client.getRenderProcessLogs()).then((logs: any[]) => {
                for (const log of logs) {
                    assert.notEqual(log.level, 'error', log.message);
                }
            });
    });
});
