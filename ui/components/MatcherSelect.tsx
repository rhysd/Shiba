import * as React from 'react';
import { useState } from 'react';
import IconButton from '@mui/material/IconButton';
import MenuItem from '@mui/material/MenuItem';
import Menu from '@mui/material/Menu';
import ManageSearchIcon from '@mui/icons-material/ManageSearch';
import type { SearchMatcher } from '../ipc';
import { type Dispatch, setSearchMatcher } from '../reducer';
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

interface Props {
    matcher: SearchMatcher;
    dispatch: Dispatch;
    onSelect?: (selected: SearchMatcher) => void;
}

export const MatcherSelect: React.FC<Props> = ({ matcher, dispatch, onSelect }) => {
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
            <IconButton
                size="small"
                title="Select search matcher"
                aria-label="select search matcher"
                onClick={handleOpen}
            >
                <ManageSearchIcon />
            </IconButton>
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
