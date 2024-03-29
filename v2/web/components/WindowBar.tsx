import * as React from 'react';
import { sendMessage } from '../ipc';

const STYLE: React.CSSProperties = {
    minWidth: '70px',
    width: '100%',
    height: '30px',
};

function detectDragStart(event: React.MouseEvent): void {
    if (event.button !== 0) {
        return;
    }
    event.preventDefault();
    sendMessage({ kind: 'drag_window' });
}

function onDoubleClick(event: React.MouseEvent): void {
    event.preventDefault();
    sendMessage({ kind: 'toggle_maximized' });
}

export const WindowBar: React.FC = () => {
    return <div style={STYLE} onMouseDown={detectDragStart} onDoubleClick={onDoubleClick} />;
};
