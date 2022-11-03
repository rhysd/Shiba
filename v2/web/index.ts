import { mount } from './components';
import { Shiba } from './shiba';
import type { MessageFromMain } from './ipc';

declare global {
    interface Window {
        postShibaMessageFromMain(msg: MessageFromMain): Promise<void>;
    }
}

const app = new Shiba();
window.postShibaMessageFromMain = app.receive.bind(app); // The main process sends events via `window.ShibaApp` global variable
mount(document.getElementById('shiba-root')!, app);
