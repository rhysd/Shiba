import React from 'react';
import { sendMessage } from '../ipc';

const BODY_STYLE: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    width: '100vw',
    height: '100vh',
    zIndex: 2,
};
const LOGO_STYLE: React.CSSProperties = {
    display: 'block',
    maxWidth: '50%',
    height: 'auto',
    cursor: 'pointer',
    filter: 'grayscale(100%) opacity(0.4)',
};

function onClick(e: React.MouseEvent<HTMLElement>): void {
    e.preventDefault();
    sendMessage({ kind: 'file_dialog' });
}

export const Welcome: React.FC = () => {
    return (
        <div style={BODY_STYLE}>
            <img alt="Open file with dialog" src="/logo.png" style={LOGO_STYLE} onClick={onClick} />
        </div>
    );
};
