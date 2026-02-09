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

// TODO: Use CSS variables to dynamically change the colorscheme and access theme colors from JavaScript and style.css.
// https://mui.com/material-ui/customization/css-theme-variables/usage/
const THEME = createTheme({ colorSchemes: { dark: true } });

interface Props {
    dispatcher: GlobalDispatcher;
}

export const App: React.FC<Props> = ({ dispatcher }) => {
    const [state, dispatch] = useReducer(reducer, INITIAL_STATE);
    const {
        previewTree,
        path,
        searching,
        searchIndex,
        matcher,
        outline,
        config,
        history,
        help,
        notifying,
        notification,
        welcome,
        headings,
    } = state;

    let searchInput;
    if (searching && !welcome) {
        searchInput = (
            <Search index={searchIndex} total={previewTree.matchCount} matcher={matcher} dispatch={dispatch} />
        );
    }

    let main;
    if (welcome) {
        main = <Welcome />;
    } else {
        main = <Preview tree={previewTree} headings={headings} path={path} dispatch={dispatch} />;
    }

    let outlineDialog;
    if (outline && !welcome) {
        outlineDialog = <Outline dispatch={dispatch} />;
    }

    let historyDialog;
    if (history.length > 0) {
        historyDialog = <History history={history} dispatch={dispatch} />;
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
                {main}
                {searchInput}
                {outlineDialog}
                {historyDialog}
                {guideDialog}
                <Notification open={notifying} content={notification} dispatch={dispatch} />
            </ConfigContext.Provider>
        </ThemeProvider>
    );
};
