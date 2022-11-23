import React, { useReducer, useEffect } from 'react';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { Search } from './Search';
import { Welcome } from './Welcome';
import { Outline } from './Outline';
import { History } from './History';
import { Guide } from './Guide';
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
    const { searching, searchIndex, matcher, previewing, outline, theme, history, files, help } = state;

    let searchInput;
    if (searching && previewing) {
        searchInput = <Search index={searchIndex} matcher={matcher} dispatch={dispatch} />;
    }

    let welcome;
    if (!previewing) {
        welcome = <Welcome />;
    }

    let outlineDialog;
    if (outline && previewing) {
        outlineDialog = <Outline dispatch={dispatch} />;
    }

    let historyDialog;
    if (history && previewing) {
        historyDialog = <History history={files} dispatch={dispatch} />;
    }

    let guideDialog;
    if (help) {
        guideDialog = <Guide shortcuts={dispatcher.keymap.shortcuts} dispatcher={dispatcher} />;
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
            {historyDialog}
            {guideDialog}
            {welcome}
        </ThemeProvider>
    );
};
