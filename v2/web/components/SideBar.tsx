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
import Typography from '@mui/material/Typography';
import { ConfigContext } from './ConfigContext';
import { MenuButton } from './MenuButton';
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
const BUTTON_LABEL_STYLE = {
    overflow: 'hidden',
    textOverflow: 'ellipsis',
};
const TOOLTIP_SLOT_PROPS = { tooltip: { style: { maxWidth: 'none', padding: '0.5rem' } } };
const MENU_BUTTON_STYLE = { margin: 'auto 0' };

interface ListHeaderProps {
    path: string | null;
}

const ListHeader: React.FC<ListHeaderProps> = ({ path }) => {
    if (path?.startsWith('\\\\?\\')) {
        path = path.slice(4); // Strip UNC path
    }
    const title = <Typography variant="body2">{path}</Typography>;
    return (
        <Box component="header" sx={LIST_HEADER_SX}>
            <Tooltip title={title} arrow slotProps={TOOLTIP_SLOT_PROPS}>
                <Button
                    variant="text"
                    color="inherit"
                    fullWidth
                    disableFocusRipple
                    startIcon={<PetsIcon />}
                    sx={LIST_HEADER_BUTTON_SX}
                    onClick={onHeaderClick}
                >
                    <span style={BUTTON_LABEL_STYLE}>{fileName(path)}</span>
                </Button>
            </Tooltip>
            <MenuButton style={MENU_BUTTON_STYLE} />
        </Box>
    );
};

const LIST_SX = {
    height: '100%',
    overflowY: 'auto',
    overscrollBehavior: 'none',
    fontSize: '0.875rem',
};
const LIST_ITEM_SX = {
    padding: '0 8px',
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
                sx={LIST_ITEM_SX}
                ref={ref}
                key={key}
            >
                <ListItemButton style={style} disableGutters>
                    <ListItemText primary={h.text} disableTypography sx={sx} />
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
