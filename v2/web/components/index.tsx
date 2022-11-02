import React from 'react';
import { createRoot } from 'react-dom/client';
import { Preview } from './Preview';

export function mount(rootElem: HTMLElement) {
    const root = createRoot(rootElem);
    root.render(<Preview/>);
}

