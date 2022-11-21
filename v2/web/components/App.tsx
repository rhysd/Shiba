import React, { useReducer, useEffect } from 'react';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { Search } from './Search';
import { Welcome } from './Welcome';
import { Outline } from './Outline';
import { sendMessage } from '../ipc';
import { INITIAL_STATE, reducer } from '../reducer';
import type { GlobalDispatcher } from '../dispatcher';

const LIGHT_THEME = createTheme({ palette: { mode: 'light' } });
const DARK_THEME = createTheme({ palette: { mode: 'dark' } });

interface Props {
    dispatcher: GlobalDispatcher;
}

export const App: React.FC<Props> = ({ dispatcher }) => {
    const [state, dispatch] = useReducer(reducer, INITIAL_STATE);
    const { searching, searchIndex, matcher, previewing, outline, theme } = state;

    let searchInput;
    if (searching && previewing && !outline) {
        searchInput = <Search index={searchIndex} matcher={matcher} dispatch={dispatch} />;
    }

    let welcome;
    if (!previewing) {
        welcome = <Welcome />;
    }

    let outlineDialog;
    if (outline && previewing && !searching) {
        outlineDialog = <Outline dispatch={dispatch} />;
    }

    useEffect(() => {
        dispatcher.setDispatch(dispatch, state);
    });
    useEffect(() => {
        sendMessage({ kind: 'init' });
    }, []); // Run only when component was mounted

    return (
        <ThemeProvider theme={theme === 'light' ? LIGHT_THEME : DARK_THEME}>
            {searchInput}
            {outlineDialog}
            {welcome}
        </ThemeProvider>
    );
};
