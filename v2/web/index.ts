import { render } from './components';
import { Shiba } from './shiba';
import type { MessageFromMain } from './ipc';

declare global {
    interface Window {
        postShibaMessageFromMain(msg: MessageFromMain): Promise<void>;
    }
}

const app = new Shiba();
window.postShibaMessageFromMain = app.receive.bind(app); // The main process sends events via `window.ShibaApp` global variable

const root = document.getElementById('shiba-root');
if (!root) {
    throw new Error('The root element to mount application is not found in DOM');
}
render(root, app);
