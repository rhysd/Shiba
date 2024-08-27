import * as React from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './components/App';
import { GlobalDispatcher } from './dispatcher';
import type { MessageFromMain } from './ipc';
import { error } from './log';

declare global {
    interface Window {
        postShibaMessageFromMain(msg: MessageFromMain): void;
    }
}

const dispatcher = new GlobalDispatcher();

// The main process will send IPC events via this global function
window.postShibaMessageFromMain = dispatcher.handleIpcMessage.bind(dispatcher);

const reactRoot = document.getElementById('shiba-root');
if (reactRoot) {
    createRoot(reactRoot).render(<App dispatcher={dispatcher} />);
} else {
    error('The root element to mount application is not found in DOM');
}
