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
import type { BoundShortcut } from '../keymaps';
import type { GlobalDispatcher } from '../dispatcher';
import { closeHelp } from '../reducer';

const KEYBIND_ROW_STYLE: React.CSSProperties = {
    cursor: 'pointer',
};

export interface Props {
    keybinds: BoundShortcut[];
    dispatcher: GlobalDispatcher;
}

export const Guide: React.FC<Props> = ({ keybinds, dispatcher }) => {
    const handleClose = (): void => {
        dispatcher.dispatch(closeHelp());
    };

    return (
        <Dialog open scroll="paper" onClose={handleClose}>
            <DialogTitle>Guide</DialogTitle>
            <DialogContent dividers>
                <TableContainer component={Paper}>
                    <Table aria-label="key shortcut table">
                        <TableHead>
                            <TableRow>
                                <TableCell>Key mappings</TableCell>
                                <TableCell>Description</TableCell>
                            </TableRow>
                        </TableHead>
                        <TableBody>
                            {keybinds.map((bind, i) => (
                                <TableRow
                                    key={i}
                                    hover
                                    onClick={() => {
                                        handleClose();
                                        bind.shortcut.dispatch(dispatcher);
                                    }}
                                    style={KEYBIND_ROW_STYLE}
                                >
                                    <TableCell>
                                        {bind.binds.map((b, j) => (
                                            <Chip key={j} size="small" label={b} variant="outlined" />
                                        ))}
                                    </TableCell>
                                    <TableCell>{bind.shortcut.description}</TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </TableContainer>
            </DialogContent>
        </Dialog>
    );
};
