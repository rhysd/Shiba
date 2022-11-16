import React, { useReducer, useEffect } from 'react';
import { Search } from './Search';
import { Welcome } from './Welcome';
import { sendMessage } from '../ipc';
import { INITIAL_STATE, reducer } from '../reducer';
import type { GlobalDispatcher } from '../dispatcher';

interface Props {
    dispatcher: GlobalDispatcher;
}

export const App: React.FC<Props> = ({ dispatcher }) => {
    const [state, dispatch] = useReducer(reducer, INITIAL_STATE);
    const { searching, searchIndex, matcher, previewing } = state;

    let searchInput;
    if (searching && previewing) {
        searchInput = <Search index={searchIndex} matcher={matcher} dispatch={dispatch} />;
    }

    let welcome;
    if (!previewing) {
        welcome = <Welcome />;
    }

    useEffect(() => {
        dispatcher.setDispatch(dispatch, state);
    });
    useEffect(() => {
        sendMessage({ kind: 'init' });
    }, []); // Run only when component was mounted

    return (
        <>
            {searchInput}
            {welcome}
        </>
    );
};
