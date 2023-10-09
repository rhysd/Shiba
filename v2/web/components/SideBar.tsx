import * as React from 'react';
import { useRef, useEffect } from 'react';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemText from '@mui/material/ListItemText';
import Button from '@mui/material/Button';
import ArticleIcon from '@mui/icons-material/Article';
import Tooltip from '@mui/material/Tooltip';
import Divider from '@mui/material/Divider';
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

const LIST_SX = {
    height: '100%',
    overflowY: 'auto',
    overscrollBehavior: 'none',
};

const HEADER_SX = {
    overflow: 'hidden',
    whiteSpace: 'nowrap',
    textOverflow: 'ellipsis',
    textTransform: 'none',
    color: 'text.primary',
    fontWeight: 'h6',
};

interface Props {
    headings: Heading[];
    path: string | null;
}

export const SideBar: React.FC<Props> = ({ headings, path }) => {
    const focusedRef = useRef<HTMLLIElement>(null);
    const listRef = useRef<HTMLUListElement>(null);
    useEffect(() => {
        if (focusedRef.current && listRef.current) {
            scrollIntoSideBar(focusedRef.current, listRef.current);
        }
    }, [headings]);

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

    return (
        <>
            <Tooltip title={path} arrow>
                <Button
                    variant="text"
                    fullWidth
                    disableFocusRipple
                    startIcon={<ArticleIcon />}
                    sx={HEADER_SX}
                    onClick={onHeaderClick}
                >
                    {fileName(path)}
                </Button>
            </Tooltip>
            <Divider />
            <List sx={LIST_SX} ref={listRef}>
                {children}
            </List>
            ;
        </>
    );
};
