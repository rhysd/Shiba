import React from 'react';
import Snackbar from '@mui/material/Snackbar';
import IconButton from '@mui/material/IconButton';
import ZoomInIcon from '@mui/icons-material/ZoomIn';
import ZoomOutIcon from '@mui/icons-material/ZoomOut';
import Paper from '@mui/material/Paper';
import { dismissNotification, type Dispatch } from '../reducer';
import { sendMessage } from '../ipc';

const ORIGIN = { vertical: 'bottom', horizontal: 'right' } as const;
const ROOT_STYLE: React.CSSProperties = {
    bottom: '0',
    right: '0',
};
const BODY_STYLE: React.CSSProperties = {
    borderTopRightRadius: '0',
    borderBottomRightRadius: '0',
    borderBottomLeftRadius: '0',
    borderRight: '0',
    borderBottom: '0',
    padding: '2px 8px',
};

function zoomIn(): void {
    sendMessage({ kind: 'zoom', zoom: 'In' });
}
function zoomOut(): void {
    sendMessage({ kind: 'zoom', zoom: 'Out' });
}

export interface ZoomProps {
    open: boolean;
    percent: number;
    dispatch: Dispatch;
}

export const ZoomNotification: React.FC<ZoomProps> = ({ open, percent, dispatch }) => {
    const handleClose = (): void => {
        dispatch(dismissNotification());
    };

    return (
        <Snackbar style={ROOT_STYLE} open={open} autoHideDuration={4000} onClose={handleClose} anchorOrigin={ORIGIN}>
            <Paper style={BODY_STYLE} variant="outlined">
                Zoom: {percent}%
                <IconButton onClick={zoomIn} size="small" aria-label="zoom in" color="info">
                    <ZoomInIcon fontSize="small" />
                </IconButton>
                <IconButton onClick={zoomOut} size="small" aria-label="zoom out" color="info">
                    <ZoomOutIcon fontSize="small" />
                </IconButton>
            </Paper>
        </Snackbar>
    );
};
