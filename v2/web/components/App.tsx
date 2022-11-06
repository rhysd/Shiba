import React, { useReducer, useEffect } from 'react';
import { Preview } from './Preview';
import { sendMessage } from '../ipc';
import { INITIAL_STATE, reducer } from '../reducer';
import type { Dispatcher } from '../dispatcher';

interface Props {
    dispatcher: Dispatcher;
}

export const App: React.FC<Props> = ({ dispatcher }) => {
    const [state, dispatch] = useReducer(reducer, INITIAL_STATE);

    useEffect(() => {
        dispatcher.setDispatch(dispatch, state);
    });
    useEffect(() => {
        sendMessage({ kind: 'init' });
    }, []); // Run only when component was mounted

    return <Preview state={state} dispatch={dispatch} />;
};
