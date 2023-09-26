import * as React from 'react';
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
import Link from '@mui/material/Link';
import { useTheme, type Theme } from '@mui/material/styles';
import type { PaperProps } from '@mui/material/Paper';
import type { BoundShortcut } from '../keymaps';
import type { GlobalDispatcher } from '../dispatcher';
import { closeHelp } from '../reducer';
import { parseColor } from '../css';

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
const BIND_CHIP_STYLE: React.CSSProperties = {
    margin: '4px',
};
const TABLE_STYLE = {
    boxSizing: 'border-box', // Prevent borders from causing the horizontal scroll bar on Windows
};

function blurBackdropProps(theme: Theme): PaperProps {
    let color = theme.palette.background.paper;
    const rgb = parseColor(color);
    if (rgb !== null) {
        color = `rgba(${rgb[0]},${rgb[1]},${rgb[2]},0.8)`; // `opacity` is not available for `backdrop-filter`
    }
    return {
        style: {
            backgroundColor: color,
            backdropFilter: 'blur(10px)',
            WebkitBackdropFilter: 'blur(10px)',
        },
    };
}

export interface Props {
    shortcuts: BoundShortcut[];
    dispatcher: GlobalDispatcher;
}

export const Guide: React.FC<Props> = ({ shortcuts, dispatcher }) => {
    const theme = useTheme();
    const handleClose = (): void => {
        dispatcher.dispatch(closeHelp());
    };
    const props = blurBackdropProps(theme);

    return (
        <Dialog open scroll="paper" onClose={handleClose} PaperProps={props}>
            <DialogTitle style={TITLE_STYLE}>
                Key Guide
                <IconButton aria-label="close" style={CLOSE_BUTTON_STYLE} onClick={handleClose}>
                    <CloseIcon />
                </IconButton>
            </DialogTitle>
            <DialogContent dividers>
                <TableContainer component={Paper} variant="outlined" sx={TABLE_STYLE}>
                    <Table aria-label="key shortcut table">
                        <caption>
                            Visit{' '}
                            <Link href="https://github.com/rhysd/Shiba/tree/main/docs">the document directory</Link> to
                            know about the usage and customization. If you're seeing some bugs, report it by{' '}
                            <Link href="https://github.com/rhysd/Shiba/issues">creating a new issue</Link>.
                        </caption>
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
                                        {shortcut.binds.map((label, key) => (
                                            <Chip key={key} size="small" label={label} style={BIND_CHIP_STYLE} />
                                        ))}
                                    </TableCell>
                                    <TableCell>{shortcut.description}</TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </TableContainer>
            </DialogContent>
        </Dialog>
    );
};
