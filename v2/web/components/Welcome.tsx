import * as React from 'react';
import { useContext } from 'react';
import Box from '@mui/material/Box';
import { WindowBar } from './WindowBar';
import { ConfigContext } from './ConfigContext';
import { sendMessage } from '../ipc';

const VIBRANT_BODY_SX = {
    width: '100%',
    height: '100%',
    zIndex: 2,
    display: 'flex',
    flexDirection: 'column',
};
const NON_VIBRANT_BODY_SX = { bgcolor: 'background.paper', ...VIBRANT_BODY_SX };
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

export const Welcome: React.FC = () => {
    const { titleBar, vibrant } = useContext(ConfigContext);
    const sx = vibrant ? VIBRANT_BODY_SX : NON_VIBRANT_BODY_SX;
    return (
        <Box sx={sx}>
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
        </Box>
    );
};
