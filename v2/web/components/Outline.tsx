import React, { useState, useRef, useEffect } from 'react';
import Dialog from '@mui/material/Dialog';
import DialogContent from '@mui/material/DialogContent';
import DialogTitle from '@mui/material/DialogTitle';
import List from '@mui/material/List';
import ListItemButton from '@mui/material/ListItemButton';
import InputBase from '@mui/material/InputBase';
import type { PaperProps } from '@mui/material/Paper';
import { type Dispatch, closeOutline } from '../reducer';

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

interface Heading {
    prefix: string;
    text: string;
    elem: HTMLHeadingElement;
    show: boolean;
}

function collectHeadings(): Heading[] {
    const ret = [];
    for (const elem of document.querySelectorAll('h1,h2,h3,h4,h5,h6') as NodeListOf<HTMLHeadingElement>) {
        const prefix = '#'.repeat(parseInt(elem.tagName.slice(1), 10));
        const text = elem.textContent ?? '';
        ret.push({ prefix, text, elem, show: true });
    }
    return ret;
}

interface Props {
    dispatch: Dispatch;
}

export const Outline: React.FC<Props> = ({ dispatch }) => {
    const [index, setIndex] = useState(0);
    const [headings, setHeadings] = useState(collectHeadings);
    const focusedItemRef = useRef<HTMLDivElement | null>(null);
    const visibleHeadings = headings.filter(h => h.show);

    useEffect(() => {
        if (focusedItemRef.current !== null) {
            // <ListItemButton>'s autoFocus prop is not available since it takes away focus from the <input/>
            focusedItemRef.current.scrollIntoView({
                behavior: 'smooth',
                block: 'nearest',
                inline: 'start',
            });
        }
    });

    const handleClose = (): void => {
        dispatch(closeOutline());
    };

    const handleItemClick = (item: Heading): void => {
        item.elem.scrollIntoView({
            behavior: 'smooth',
            block: 'start',
            inline: 'start',
        });
        dispatch(closeOutline());
    };

    const handleInput = (e: React.FormEvent<HTMLInputElement>): void => {
        const input = e.currentTarget.value.toLowerCase();
        if (input.length === 0) {
            setHeadings(headings.map(h => ({ ...h, show: true })));
            return;
        }

        setHeadings(
            headings.map(h => {
                const text = h.text.toLowerCase();
                return { ...h, show: text.includes(input) };
            }),
        );
    };

    const handleKeydown = (e: React.KeyboardEvent<HTMLInputElement>): void => {
        if (
            (e.key === 'n' && !e.shiftKey && e.ctrlKey) ||
            (e.key === 'ArrowDown' && !e.ctrlKey) ||
            (e.key === 'Tab' && !e.shiftKey)
        ) {
            let next = index + 1;
            if (next >= visibleHeadings.length) {
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
                next = Math.max(visibleHeadings.length - 1, 0); // wrap
            }
            setIndex(next);
        } else if (e.key === 'ArrowDown' && e.ctrlKey) {
            setIndex(Math.max(visibleHeadings.length - 1, 0));
        } else if (e.key === 'ArrowUp' && e.ctrlKey) {
            setIndex(0);
        } else if (e.key === 'Enter') {
            if (index < visibleHeadings.length) {
                handleItemClick(visibleHeadings[index]);
            }
        } else {
            return;
        }
        e.preventDefault();
    };

    return (
        <Dialog PaperProps={PAPER_PROPS} onClose={handleClose} open>
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
                    {visibleHeadings.map((item, idx) => {
                        const selected = index === idx;
                        const ref = index === idx ? focusedItemRef : undefined;
                        return (
                            <ListItemButton
                                selected={selected}
                                onClick={() => {
                                    handleItemClick(item);
                                }}
                                ref={ref}
                                key={idx}
                            >
                                {item.prefix} {item.text}
                            </ListItemButton>
                        );
                    })}
                </List>
            </DialogContent>
        </Dialog>
    );
};
