import React from 'react';
import mermaid from 'mermaid';
import { useTheme } from '@mui/material/styles';
import * as log from '../log';

let id = 0;
function getId(): number {
    if (id >= Number.MAX_SAFE_INTEGER) {
        id = Number.MIN_SAFE_INTEGER;
    } else {
        id++;
    }
    return id;
}

let initialized = false;
function initialize(theme: 'dark' | 'default'): void {
    if (initialized) {
        return;
    }
    mermaid.initialize({ startOnLoad: false, theme });
    log.debug('Initialized mermaid renderer', theme);
    initialized = true;
}

export interface Props {
    content: string;
}

export const Mermaid: React.FC<Props> = ({ content }) => {
    const theme = useTheme();
    initialize(theme.palette.mode === 'dark' ? 'dark' : 'default');
    const svg = mermaid.render(`graph-${getId()}`, content);
    return <div className="mermaid" dangerouslySetInnerHTML={{ __html: svg }} />; // eslint-disable-line @typescript-eslint/naming-convention
};
