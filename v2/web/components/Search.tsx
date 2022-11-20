import React, { useEffect, useRef, useState } from 'react';
import Paper from '@mui/material/Paper';
import IconButton from '@mui/material/IconButton';
import InputBase from '@mui/material/InputBase';
import Divider from '@mui/material/Divider';
import KeyboardArrowUpIcon from '@mui/icons-material/KeyboardArrowUp';
import KeyboardArrowDownIcon from '@mui/icons-material/KeyboardArrowDown';
import CloseIcon from '@mui/icons-material/Close';
import Typography from '@mui/material/Typography';
import { MatcherSelect } from './MatcherSelect';
import { type Dispatch, searchNext, searchPrevious, closeSearch } from '../reducer';
import { countSearchMatches } from '../search';
import { type SearchMatcher, sendMessage } from '../ipc';

const DEBOUNCE_TIMEOUT = 100; // 100ms
const PAPER_STYLE: React.CSSProperties = {
    position: 'fixed',
    top: 0,
    right: 0,
    width: '420px',
    margin: '8px',
    padding: '8px',
    display: 'flex',
    alignItems: 'center',
};
const COUNTER_STYLE: React.CSSProperties = {
    maxWidth: '200px',
    cursor: 'default',
};
const INPUT_STYLE: React.CSSProperties = {
    flex: 'auto',
    marginLeft: '4px',
    fontFamily: 'inherit',
};
const DIVIDER_STYLE: React.CSSProperties = {
    marginLeft: '0.5rem',
    height: '1.5rem',
};

interface Props {
    index: number | null;
    matcher: SearchMatcher;
    dispatch: Dispatch;
}

export const Search: React.FC<Props> = ({ index, matcher, dispatch }) => {
    const counterElem = useRef<HTMLDivElement | null>(null);
    const inputElem = useRef<HTMLInputElement | null>(null);
    const [debId, setDebId] = useState<number | null>(null);

    useEffect(() => {
        if (counterElem.current !== null) {
            const nth = index !== null ? index + 1 : 0;
            const total = countSearchMatches();
            counterElem.current.textContent = `${nth} / ${total}`;
        }
    }, [index]);

    const handlePrev = (): void => {
        dispatch(searchPrevious(index));
    };
    const handleNext = (): void => {
        dispatch(searchNext(index));
    };
    const handleClose = (): void => {
        sendMessage({ kind: 'search', query: '', index: null, matcher });
        dispatch(closeSearch());
    };
    const handleChange = (e: React.FormEvent<HTMLInputElement>): void => {
        if (debId !== null) {
            clearTimeout(debId);
        }
        const query = e.currentTarget.value;
        const id = setTimeout(() => {
            sendMessage({ kind: 'search', query, index, matcher });
            setDebId(null);
        }, DEBOUNCE_TIMEOUT);
        setDebId(id);
    };
    const handleKeydown = (e: React.KeyboardEvent<HTMLInputElement>): void => {
        if (e.key === 'Enter' && !e.shiftKey) {
            handleNext();
        } else if (e.key === 'Enter' && e.shiftKey) {
            handlePrev();
        } else if (e.key === 'Escape') {
            handleClose();
        } else {
            return;
        }
        e.preventDefault();
    };
    const focusInputElem = (): void => {
        // Focus <input> at next tick since re-render will happen after this callback and it will blur the element again
        setTimeout(() => inputElem.current?.focus(), 0);
    };

    return (
        <Paper elevation={4} style={PAPER_STYLE}>
            <MatcherSelect matcher={matcher} dispatch={dispatch} onSelect={focusInputElem} />
            <InputBase
                style={INPUT_STYLE}
                inputProps={{
                    'aria-label': 'search input',
                    onChange: handleChange,
                    onKeyDown: handleKeydown,
                    style: { padding: 0 },
                    ref: inputElem,
                }}
                type="search"
                placeholder="Search…"
                autoFocus
            />
            <Typography style={COUNTER_STYLE} ref={counterElem}></Typography>
            <Divider style={DIVIDER_STYLE} orientation="vertical" />
            <IconButton size="small" title="Find backward" aria-label="find backward" onClick={handlePrev}>
                <KeyboardArrowUpIcon fontSize="small" />
            </IconButton>
            <IconButton size="small" title="Find forward" aria-label="find forward" onClick={handleNext}>
                <KeyboardArrowDownIcon fontSize="small" />
            </IconButton>
            <IconButton size="small" title="Close search" aria-label="close search" onClick={handleClose}>
                <CloseIcon fontSize="small" />
            </IconButton>
        </Paper>
    );
};
