import { mount } from './components';
import { Shiba } from './shiba';

declare global {
    interface Window {
        ShibaApp: Shiba;
    }
}

const app = new Shiba();
window.ShibaApp = app; // The main process sends events via `window.ShibaApp` global variable
mount(document.getElementById('shiba-root')!);
