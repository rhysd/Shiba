import React from 'react';
import Snackbar from '@mui/material/Snackbar';
import SnackbarContent from '@mui/material/SnackbarContent';
import IconButton from '@mui/material/IconButton';
import CloseIcon from '@mui/icons-material/Close';
import ZoomInIcon from '@mui/icons-material/ZoomIn';
import ZoomOutIcon from '@mui/icons-material/ZoomOut';
import { dismissNotification, type Dispatch } from '../reducer';
import { sendMessage } from '../ipc';

const ORIGIN = { vertical: 'bottom', horizontal: 'right' } as const;

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
    const action = (
        <>
            <IconButton onClick={zoomIn} size="small" aria-label="zoom in" color="info">
                <ZoomInIcon fontSize="small" />
            </IconButton>
            <IconButton onClick={zoomOut} size="small" aria-label="zoom out" color="info">
                <ZoomOutIcon fontSize="small" />
            </IconButton>
            <IconButton onClick={handleClose} size="small" aria-label="close" color="inherit">
                <CloseIcon fontSize="small" />
            </IconButton>
        </>
    );

    // Keep font size when scale factor is smaller than 1.0
    const fontSize = open && percent < 100 ? `${100 / percent}em` : '1em';

    return (
        <Snackbar open={open} autoHideDuration={4000} onClose={handleClose} anchorOrigin={ORIGIN}>
            <SnackbarContent style={{ fontSize }} message={`Zoom: ${percent}%`} action={action} />
        </Snackbar>
    );
};
