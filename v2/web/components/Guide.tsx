import React from 'react';
import Dialog from '@mui/material/Dialog';
import DialogContent from '@mui/material/DialogContent';
import DialogTitle from '@mui/material/DialogTitle';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import Paper from '@mui/material/Paper';
import Chip from '@mui/material/Chip';
import IconButton from '@mui/material/IconButton';
import CloseIcon from '@mui/icons-material/Close';
import Typography from '@mui/material/Typography';
import Link from '@mui/material/Link';
import type { BoundShortcut } from '../keymaps';
import type { GlobalDispatcher } from '../dispatcher';
import { closeHelp } from '../reducer';

const KEYBIND_ROW_STYLE: React.CSSProperties = {
    cursor: 'pointer',
};
const TITLE_STYLE: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'row',
};
const CLOSE_BUTTON_STYLE: React.CSSProperties = {
    marginLeft: 'auto',
};
const BOTTOM_DESC_STYLE: React.CSSProperties = {
    marginTop: '16px',
};
const BIND_CHIP_STYLE: React.CSSProperties = {
    marginLeft: '4px',
};

export interface Props {
    shortcuts: BoundShortcut[];
    dispatcher: GlobalDispatcher;
}

export const Guide: React.FC<Props> = ({ shortcuts, dispatcher }) => {
    const handleClose = (): void => {
        dispatcher.dispatch(closeHelp());
    };

    return (
        <Dialog open scroll="paper" onClose={handleClose}>
            <DialogTitle style={TITLE_STYLE}>
                Guide
                <IconButton aria-label="close" style={CLOSE_BUTTON_STYLE} onClick={handleClose}>
                    <CloseIcon />
                </IconButton>
            </DialogTitle>
            <DialogContent dividers>
                <TableContainer component={Paper} variant="outlined">
                    <Table aria-label="key shortcut table">
                        <TableHead>
                            <TableRow>
                                <TableCell>Key mappings</TableCell>
                                <TableCell>Description</TableCell>
                            </TableRow>
                        </TableHead>
                        <TableBody>
                            {shortcuts.map((shortcut, i) => (
                                <TableRow
                                    key={i}
                                    hover
                                    onClick={() => {
                                        handleClose();
                                        shortcut.dispatch(dispatcher);
                                    }}
                                    style={KEYBIND_ROW_STYLE}
                                >
                                    <TableCell>
                                        {shortcut.binds.map((b, j) => (
                                            <Chip
                                                key={j}
                                                size="small"
                                                label={b}
                                                style={j === 0 ? {} : BIND_CHIP_STYLE}
                                            />
                                        ))}
                                    </TableCell>
                                    <TableCell>{shortcut.description}</TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </TableContainer>
                <Typography paragraph style={BOTTOM_DESC_STYLE}>
                    Visit <Link href="https://github.com/rhysd/Shiba/tree/master/docs">the document directory</Link> to
                    know about usage and customization. If you're seeing some bugs, report it by{' '}
                    <Link href="https://github.com/rhysd/Shiba/issues">creating a new issue</Link>.
                </Typography>
            </DialogContent>
        </Dialog>
    );
};
