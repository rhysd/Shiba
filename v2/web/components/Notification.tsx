import React from 'react';
import Snackbar from '@mui/material/Snackbar';
import SnackbarContent from '@mui/material/SnackbarContent';
import IconButton from '@mui/material/IconButton';
import CloseIcon from '@mui/icons-material/Close';
import { dismissNotification, type Dispatch } from '../reducer';

const ORIGIN = { vertical: 'bottom', horizontal: 'right' } as const;

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
        <IconButton onClick={handleClose} size="small" aria-label="close" color="inherit">
            <CloseIcon fontSize="small" />
        </IconButton>
    );

    // Keep font size when scale factor is smaller than 1.0
    const fontSize = open && percent < 100 ? `${100 / percent}em` : '1em';

    return (
        <Snackbar open={open} autoHideDuration={4000} onClose={handleClose} anchorOrigin={ORIGIN}>
            <SnackbarContent style={{ fontSize }} message={`Zoom: ${percent}%`} action={action} />
        </Snackbar>
    );
};
