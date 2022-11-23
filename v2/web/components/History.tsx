import React, { useMemo, useCallback } from 'react';
import { Palette } from './Palette';
import { type Dispatch, closeHistory } from '../reducer';
import { sendMessage } from '../ipc';
import * as log from '../log';

interface HistoryItem {
    text: string;
}

export interface Props {
    history: string[];
    dispatch: Dispatch;
}

export const History: React.FC<Props> = ({ history, dispatch }) => {
    const items = useMemo(() => {
        const items = history.map(path => ({ text: path }));
        return items.reverse();
    }, [history]);

    const handleClose = useCallback(() => {
        dispatch(closeHistory());
    }, [dispatch]);

    const handleSelect = useCallback(
        (item: HistoryItem) => {
            log.debug('Opening file via history:', item.text);
            sendMessage({ kind: 'open_file', path: item.text });
            dispatch(closeHistory());
        },
        [dispatch],
    );

    return <Palette items={items} placeholder="Search history…" onClose={handleClose} onSelect={handleSelect} />;
};
