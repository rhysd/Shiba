import { Preview } from './Preview';
import React, { useReducer, useEffect } from 'react';
import { sendMessage } from '../ipc';
import { INITIAL_STATE, Dispatcher, reducer } from '../reducer';

interface Props {
    dispatcher: Dispatcher;
}

export const App: React.FC<Props> = ({ dispatcher }) => {
    const [state, dispatch] = useReducer(reducer, INITIAL_STATE);

    useEffect(() => dispatcher.setDispatch(dispatch));
    useEffect(() => {
        sendMessage({ kind: 'init' });
    }, []); // Run only when component was mounted

    return <Preview state={state} dispatch={dispatch} />;
};
