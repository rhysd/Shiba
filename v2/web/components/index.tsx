import React from 'react';
import { createRoot } from 'react-dom/client';
import { Preview } from './Preview';
import { ShibaContext } from './context';
import type { Shiba } from '../shiba';

export function mount(rootElem: HTMLElement, app: Shiba) {
    const root = createRoot(rootElem);
    root.render(
        <ShibaContext.Provider value={app}>
            <Preview />
        </ShibaContext.Provider>,
    );
}
