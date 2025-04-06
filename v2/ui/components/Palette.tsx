import * as React from 'react';
import { useState, useRef, useEffect } from 'react';
import Dialog from '@mui/material/Dialog';
import DialogContent from '@mui/material/DialogContent';
import DialogTitle from '@mui/material/DialogTitle';
import List from '@mui/material/List';
import ListItemButton from '@mui/material/ListItemButton';
import InputBase from '@mui/material/InputBase';
import { useTheme, type Theme } from '@mui/material/styles';
import type { PaperProps } from '@mui/material/Paper';
import { parseColor } from '../css';

const CONTENT_STYLE: React.CSSProperties = {
    padding: '8px 0',
};

function bodyProps(theme: Theme): PaperProps {
    let color = theme.palette.background.paper;
    const rgb = parseColor(color);
    if (rgb !== null) {
        color = `rgba(${rgb[0]},${rgb[1]},${rgb[2]},0.8)`; // `opacity` is not available for `backdrop-filter`
    }
    return {
        style: {
            position: 'fixed', // Fix y-position on narrowing down
            margin: '32px auto',
            top: '0',
            minWidth: '60%',
            backgroundColor: color,
            backdropFilter: 'blur(10px)',
            WebkitBackdropFilter: 'blur(10px)',
        },
    };
}

export interface Item {
    text: string;
}

export interface Props<T extends Item> {
    items: T[];
    placeholder: string;
    onClose: () => void;
    onSelect: (item: T) => void;
    renderItem: (item: T) => React.ReactNode;
}

export function Palette<T extends Item>({
    items,
    placeholder,
    onClose,
    onSelect,
    renderItem,
}: Props<T>): React.ReactElement {
    const theme: Theme = useTheme();
    const [query, setQuery] = useState('');
    const [unadjustedIndex, setIndex] = useState(0);
    const focusedItemRef = useRef<HTMLDivElement>(null);
    items = query === '' ? items : items.filter(h => h.text.toLowerCase().includes(query));
    const index = unadjustedIndex < items.length ? unadjustedIndex : items.length > 0 ? items.length - 1 : 0;

    useEffect(() => {
        // <ListItemButton>'s autoFocus prop is not available since it takes away focus from the <input/>
        if (focusedItemRef.current !== null) {
            const block = index === 0 || index === items.length - 1 ? 'center' : 'nearest';
            focusedItemRef.current.scrollIntoView({
                behavior: 'smooth',
                block,
                inline: 'start',
            });
        }
    }, [index, items]);

    const handleInput = (e: React.FormEvent<HTMLInputElement>): void => {
        setQuery(e.currentTarget.value.toLowerCase());
        e.preventDefault();
    };

    const handleKeydown = (e: React.KeyboardEvent<HTMLInputElement>): void => {
        if (
            (e.key === 'n' && !e.shiftKey && e.ctrlKey) ||
            (e.key === 'ArrowDown' && !e.ctrlKey) ||
            (e.key === 'Tab' && !e.shiftKey)
        ) {
            let next = index + 1;
            if (next >= items.length) {
                next = 0; // wrap
            }
            setIndex(next);
        } else if (
            (e.key === 'p' && !e.shiftKey && e.ctrlKey) ||
            (e.key === 'ArrowUp' && !e.ctrlKey) ||
            (e.key === 'Tab' && e.shiftKey)
        ) {
            let next = index - 1;
            if (next < 0) {
                next = Math.max(items.length - 1, 0); // wrap
            }
            setIndex(next);
        } else if (e.key === 'ArrowDown' && e.ctrlKey) {
            setIndex(Math.max(items.length - 1, 0));
        } else if (e.key === 'ArrowUp' && e.ctrlKey) {
            setIndex(0);
        } else if (e.key === 'Enter') {
            if (index < items.length) {
                onSelect(items[index]);
            }
        } else {
            return;
        }
        e.preventDefault();
    };

    // `keydown` event is not triggered on WKWebView only when Escape is pressed while focusing on <input>
    const handleKeyup = (e: React.KeyboardEvent<HTMLInputElement>): void => {
        if (e.key === 'Escape') {
            e.preventDefault();

            // Note: This `blur()` call is a workaround for Safari.
            // Safari has a bug to scroll the page to the input element automatically.
            // To prevent this issue, removing the focus before unmounting is needed.
            // https://github.com/sweetalert2/sweetalert2/issues/2088
            e.currentTarget.blur();

            onClose();
        }
    };

    return (
        <Dialog slotProps={{ paper: bodyProps(theme) }} onClose={onClose} open scroll="paper">
            <DialogTitle>
                <InputBase
                    inputProps={{
                        'aria-label': 'search outline',
                        onChange: handleInput,
                        onKeyDown: handleKeydown,
                        onKeyUp: handleKeyup,
                        style: { padding: '0' },
                    }}
                    type="search"
                    placeholder={placeholder}
                    autoFocus
                    fullWidth
                />
            </DialogTitle>
            <DialogContent style={CONTENT_STYLE} dividers>
                <List>
                    {items.map((item, idx) => {
                        const selected = index === idx;
                        const ref = selected ? focusedItemRef : undefined;
                        return (
                            <ListItemButton
                                selected={selected}
                                onClick={() => {
                                    onSelect(item);
                                }}
                                ref={ref}
                                key={idx}
                            >
                                {renderItem(item)}
                            </ListItemButton>
                        );
                    })}
                </List>
            </DialogContent>
        </Dialog>
    );
}
