import React, { useState } from 'react';
import MenuItem from '@mui/material/MenuItem';
import Menu from '@mui/material/Menu';
import Button from '@mui/material/Button';
import type { SearchMatcher } from '../ipc';
import { Dispatch, setSearchMatcher } from '../reducer';
import * as log from '../log';

const MENU_ITEM_STYLE: React.CSSProperties = {
    fontFamily: 'inherit',
    fontSize: '0.8rem',
};

const ALL_MATCHERS: [SearchMatcher, string][] = [
    ['SmartCase', 'smart case'],
    ['CaseSensitive', 'case sensitive'],
    ['CaseInsensitive', 'case insensitive'],
    ['CaseSensitiveRegex', 'regular expression'],
];
const BUTTON_STYLE: React.CSSProperties = {
    textTransform: 'none',
    backgroundColor: 'inherit',
    fontSize: '1rem',
    padding: '4px',
    minWidth: '3rem',
};

interface IconProps {
    matcher: SearchMatcher;
    onClick: (e: React.MouseEvent<HTMLElement>) => void;
}

const MatcherIcon: React.FC<IconProps> = ({ matcher, onClick }) => (
    <Button style={BUTTON_STYLE} onClick={onClick} title="Select search matcher" aria-label="select search matcher">
        {matcher === 'SmartCase'
            ? 'a→A'
            : matcher === 'CaseSensitive'
            ? 'a≠A'
            : matcher === 'CaseInsensitive'
            ? 'a=A'
            : matcher === 'CaseSensitiveRegex'
            ? '/aA/'
            : '???'}
    </Button>
);

interface SelectProps {
    matcher: SearchMatcher;
    dispatch: Dispatch;
    onSelect?: (selected: SearchMatcher) => void;
}

export const MatcherSelect: React.FC<SelectProps> = ({ matcher, dispatch, onSelect }) => {
    const [anchor, setAnchor] = useState<HTMLElement | null>(null);

    const handleOpen = (e: React.MouseEvent<HTMLElement>): void => {
        setAnchor(e.currentTarget);
    };
    const handleClose = (): void => {
        setAnchor(null);
    };
    const handleSelect = (selected: SearchMatcher): void => {
        log.debug('Search matcher selected', selected);
        if (selected !== matcher) {
            dispatch(setSearchMatcher(selected));
        }
        setAnchor(null);
        onSelect?.(selected);
    };

    return (
        <>
            <MatcherIcon matcher={matcher} onClick={handleOpen} />
            <Menu anchorEl={anchor} open={anchor !== null} onClose={handleClose}>
                {ALL_MATCHERS.map(([m, desc]) => (
                    <MenuItem
                        key={m}
                        style={MENU_ITEM_STYLE}
                        selected={matcher === m}
                        onClick={() => {
                            handleSelect(m);
                        }}
                    >
                        {desc}
                    </MenuItem>
                ))}
            </Menu>
        </>
    );
};
