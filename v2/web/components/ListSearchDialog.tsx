import React, { useState, useRef, useEffect } from 'react';
import Dialog from '@mui/material/Dialog';
import DialogContent from '@mui/material/DialogContent';
import DialogTitle from '@mui/material/DialogTitle';
import List from '@mui/material/List';
import ListItemButton from '@mui/material/ListItemButton';
import InputBase from '@mui/material/InputBase';
import type { PaperProps } from '@mui/material/Paper';

const CONTENT_STYLE: React.CSSProperties = {
    padding: '8px 0',
};

const PAPER_PROPS: PaperProps = {
    style: {
        // Fix y-position on narrowing down
        position: 'fixed',
        margin: '32px auto',
        top: '0',
        minWidth: '60%',
    },
};

export interface Item {
    prefix?: string;
    text: string;
}

export interface Props<T extends Item> {
    items: T[];
    onClose: () => void;
    onSelect: (item: T) => void;
}

export function ListSearchDialog<T extends Item>({ items, onClose, onSelect }: Props<T>): React.ReactElement {
    const [query, setQuery] = useState('');
    const [unadjustedIndex, setIndex] = useState(0);
    const focusedItemRef = useRef<HTMLDivElement | null>(null);
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
            if (next <= 0) {
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

    return (
        <Dialog PaperProps={PAPER_PROPS} onClose={onClose} open>
            <DialogTitle>
                <InputBase
                    inputProps={{
                        'aria-label': 'search outline',
                        onChange: handleInput,
                        onKeyDown: handleKeydown,
                        style: { padding: 0 },
                    }}
                    type="search"
                    placeholder="Search outline…"
                    autoFocus
                    fullWidth
                />
            </DialogTitle>
            <DialogContent style={CONTENT_STYLE} dividers>
                <List>
                    {items.map((item, idx) => {
                        const selected = index === idx;
                        const ref = index === idx ? focusedItemRef : undefined;
                        const text = item.prefix === undefined ? item.text : `${item.prefix} ${item.text}`;
                        return (
                            <ListItemButton
                                selected={selected}
                                onClick={() => {
                                    onSelect(item);
                                }}
                                ref={ref}
                                key={idx}
                            >
                                {text}
                            </ListItemButton>
                        );
                    })}
                </List>
            </DialogContent>
        </Dialog>
    );
}
