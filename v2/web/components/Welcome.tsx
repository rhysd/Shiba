import * as React from 'react';
import { WindowBar } from './WindowBar';
import { sendMessage } from '../ipc';

const CONTAINER_STYLE: React.CSSProperties = {
    width: '100%',
    height: '100%',
    zIndex: 2,
    display: 'flex',
    flexDirection: 'column',
};
const BODY_STYLE: React.CSSProperties = {
    flexGrow: 1,
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    width: '100%',
};
const LOGO_STYLE: React.CSSProperties = {
    display: 'block',
    maxWidth: '50%',
    height: 'auto',
    cursor: 'pointer',
    filter: 'grayscale(100%) opacity(0.4)',
    userSelect: 'none',
    WebkitUserSelect: 'none',
};

function onClick(e: React.MouseEvent<HTMLElement>): void {
    e.preventDefault();
    sendMessage({ kind: 'file_dialog' });
}

interface Props {
    titleBar: boolean;
}

export const Welcome: React.FC<Props> = ({ titleBar }) => {
    return (
        <div style={CONTAINER_STYLE}>
            {titleBar && <WindowBar />}
            <div style={BODY_STYLE}>
                <img
                    alt="Open file with dialog"
                    src="/logo.png"
                    style={LOGO_STYLE}
                    onClick={onClick}
                    draggable="false"
                />
            </div>
        </div>
    );
};
