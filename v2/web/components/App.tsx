import * as React from 'react';
import { useReducer, useEffect } from 'react';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { Preview } from './Preview';
import { Search } from './Search';
import { Welcome } from './Welcome';
import { Outline } from './Outline';
import { History } from './History';
import { Guide } from './Guide';
import { Notification } from './Notification';
import { ConfigContext } from './ConfigContext';
import { sendMessage } from '../ipc';
import { INITIAL_STATE, reducer } from '../reducer';
import type { GlobalDispatcher } from '../dispatcher';

// Note: `CssBaseline` is not available since it sets `background-color` and prevents vibrant window.

const THEME = createTheme({ colorSchemes: { dark: true } });

interface Props {
    dispatcher: GlobalDispatcher;
}

export const App: React.FC<Props> = ({ dispatcher }) => {
    const [state, dispatch] = useReducer(reducer, INITIAL_STATE);
    const {
        previewTree,
        searching,
        searchIndex,
        matcher,
        outline,
        config,
        history,
        files,
        help,
        notifying,
        notification,
        welcome,
        headings,
        currentPath,
        theme,
    } = state;

    let searchInput;
    if (searching && !welcome) {
        searchInput = (
            <Search index={searchIndex} total={previewTree.matchCount} matcher={matcher} dispatch={dispatch} />
        );
    }

    let welcomePage;
    if (welcome) {
        welcomePage = <Welcome />;
    }

    let outlineDialog;
    if (outline && !welcome) {
        outlineDialog = <Outline dispatch={dispatch} />;
    }

    let historyDialog;
    if (history) {
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
        <ThemeProvider theme={THEME}>
            <ConfigContext.Provider value={config}>
                <Preview tree={previewTree} headings={headings} path={currentPath} theme={theme} dispatch={dispatch} />
                {searchInput}
                {outlineDialog}
                {historyDialog}
                {guideDialog}
                {welcomePage}
                <Notification open={notifying} content={notification} dispatch={dispatch} />
            </ConfigContext.Provider>
        </ThemeProvider>
    );
};
