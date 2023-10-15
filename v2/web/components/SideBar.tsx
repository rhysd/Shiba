import * as React from 'react';
import { useRef, useEffect, useContext } from 'react';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemText from '@mui/material/ListItemText';
import Button from '@mui/material/Button';
import PetsIcon from '@mui/icons-material/Pets';
import Tooltip from '@mui/material/Tooltip';
import Divider from '@mui/material/Divider';
import Box from '@mui/material/Box';
import IconButton from '@mui/material/IconButton';
import MoreVertIcon from '@mui/icons-material/MoreVert';
import { ConfigContext } from './ConfigContext';
import type { Heading } from '../reducer';
import { sendMessage } from '../ipc';

function scrollIntoSideBar(focused: HTMLLIElement, list: HTMLUListElement): void {
    const needle = focused.getBoundingClientRect();
    const heystack = list.getBoundingClientRect();
    if (heystack.bottom <= needle.bottom && needle.top <= heystack.top) {
        return; // The item is already in the list
    }
    focused.scrollIntoView({
        behavior: 'smooth', // This does not work on WKWebView
        block: 'nearest',
        inline: 'nearest',
    });
}

function fileName(path: string | null): string {
    if (path === null) {
        return '';
    }
    for (const sep of ['/', '\\']) {
        const i = path.lastIndexOf(sep);
        if (i >= 0) {
            return path.slice(i + 1);
        }
    }
    return path;
}

function onHeaderClick(e: React.MouseEvent<HTMLElement>): void {
    e.preventDefault();
    sendMessage({ kind: 'file_dialog' });
}

function onMoreButtonClick(e: React.MouseEvent<HTMLElement>): void {
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    const x = rect.x + rect.width;
    const y = rect.y;
    sendMessage({ kind: 'open_menu', position: [x, y] });
}

const LIST_HEADER_BUTTON_SX = {
    overflow: 'hidden',
    whiteSpace: 'nowrap',
    textOverflow: 'ellipsis',
    textTransform: 'none',
    color: 'text.primary',
    fontWeight: 'h6',
};

const LIST_HEADER_SX = {
    display: 'flex',
    flexDirection: 'row',
};

interface ListHeaderProps {
    path: string | null;
}

const ListHeader: React.FC<ListHeaderProps> = ({ path }) => {
    if (path !== null && path.startsWith('\\\\?\\')) {
        path = path.slice(4); // Strip UNC path
    }
    return (
        <Box component="header" sx={LIST_HEADER_SX}>
            <Tooltip title={path} arrow>
                <Button
                    variant="text"
                    fullWidth
                    disableFocusRipple
                    startIcon={<PetsIcon />}
                    sx={LIST_HEADER_BUTTON_SX}
                    onClick={onHeaderClick}
                >
                    {fileName(path)}
                </Button>
            </Tooltip>
            <IconButton onClick={onMoreButtonClick} size="small" disableFocusRipple>
                <MoreVertIcon />
            </IconButton>
        </Box>
    );
};

const LIST_SX = {
    height: '100%',
    overflowY: 'auto',
    overscrollBehavior: 'none',
};

interface Props {
    headings: Heading[];
    path: string | null;
}

export const SideBar: React.FC<Props> = ({ headings, path }) => {
    const { hideScrollBar } = useContext(ConfigContext);

    const focusedRef = useRef<HTMLLIElement>(null);
    const listRef = useRef<HTMLUListElement>(null);

    useEffect(() => {
        if (focusedRef.current && listRef.current) {
            scrollIntoSideBar(focusedRef.current, listRef.current);
        }
    }, [headings]);

    useEffect(() => {
        if (!hideScrollBar || !listRef.current) {
            return;
        }
        const callback = (): void => {
            listRef.current?.classList.toggle('hide-scrollbar');
        };
        listRef.current.addEventListener('mouseenter', callback);
        listRef.current.addEventListener('mouseleave', callback);
    }, [hideScrollBar]);

    const children = headings.map((h, key) => {
        const selected = !!h.current;
        const ref = selected ? focusedRef : undefined;
        const style = {
            padding: `0 0 0 ${h.level - 1}em`,
        };
        const sx = {
            color: selected ? 'text.primary' : 'text.secondary',
        };
        return (
            <ListItem
                selected={selected}
                alignItems="flex-start"
                onClick={() => {
                    h.elem.scrollIntoView({
                        behavior: 'smooth', // This does not work on WKWebView
                        block: 'start',
                        inline: 'start',
                    });
                }}
                disablePadding
                style={{ padding: '0 8px' }}
                ref={ref}
                key={key}
            >
                <ListItemButton style={style} disableGutters>
                    <ListItemText primary={h.text} sx={sx} />
                </ListItemButton>
            </ListItem>
        );
    });
    const className = hideScrollBar ? 'hide-scrollbar' : '';

    return (
        <>
            <ListHeader path={path} />
            <Divider />
            <List className={className} sx={LIST_SX} ref={listRef}>
                {children}
            </List>
        </>
    );
};
