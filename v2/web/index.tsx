import React from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './components/App';
import { GlobalDispatcher } from './dispatcher';
import type { MessageFromMain } from './ipc';

declare global {
    interface Window {
        postShibaMessageFromMain(msg: MessageFromMain): void;
    }
}

const previewRoot = document.getElementById('preview-root');
if (!previewRoot) {
    throw new Error('The root element to mount Markdown preview is not found in DOM');
}

const dispatcher = new GlobalDispatcher(window, previewRoot);

// The main process will send IPC events via this global function
window.postShibaMessageFromMain = dispatcher.handleIpcMessage.bind(dispatcher);

const reactRoot = document.getElementById('shiba-root');
if (!reactRoot) {
    throw new Error('The root element to mount application is not found in DOM');
}

createRoot(reactRoot).render(<App dispatcher={dispatcher} />);
