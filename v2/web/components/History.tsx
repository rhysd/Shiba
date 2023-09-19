import * as React from 'react';
import { useMemo, useCallback } from 'react';
import { Palette } from './Palette';
import { type Dispatch, closeHistory } from '../reducer';
import { sendMessage } from '../ipc';
import * as log from '../log';

interface HistoryItem {
    text: string;
    path: string;
}

export interface Props {
    history: string[];
    dispatch: Dispatch;
}

function renderHistoryItem(item: HistoryItem): React.ReactNode {
    return item.text;
}

function stripUncPath(path: string): string {
    if (path.startsWith('\\\\?\\')) {
        path = path.slice(4);
    }
    return path;
}

export const History: React.FC<Props> = ({ history, dispatch }) => {
    const items = useMemo(() => {
        const items = history.map(path => ({ text: stripUncPath(path), path }));
        return items.reverse();
    }, [history]);

    const handleClose = useCallback(() => {
        dispatch(closeHistory());
    }, [dispatch]);

    const handleSelect = useCallback(
        ({ path }: HistoryItem) => {
            log.debug('Opening file via history:', path);
            sendMessage({ kind: 'open_file', path: path });
            dispatch(closeHistory());
        },
        [dispatch],
    );

    return (
        <Palette
            items={items}
            placeholder="Search history…"
            onClose={handleClose}
            onSelect={handleSelect}
            renderItem={renderHistoryItem}
        />
    );
};
