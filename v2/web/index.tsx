import React from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './components/App';
import { Dispatcher } from './dispatcher';
import type { MessageFromMain } from './ipc';

declare global {
    interface Window {
        postShibaMessageFromMain(msg: MessageFromMain): void;
    }
}

const dispatcher = new Dispatcher();

// The main process will send IPC events via this global function
window.postShibaMessageFromMain = dispatcher.handleIpcMessage.bind(dispatcher);

const root = document.getElementById('shiba-root');
if (!root) {
    throw new Error('The root element to mount application is not found in DOM');
}

createRoot(root).render(<App dispatcher={dispatcher} />);
