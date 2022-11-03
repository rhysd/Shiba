import React from 'react';
import { createRoot } from 'react-dom/client';
import { Preview } from './Preview';
import type { Shiba } from '../shiba';

export function render(rootElem: HTMLElement, app: Shiba) {
    const root = createRoot(rootElem);
    root.render(
        <Preview app={app}/>
    );
}
