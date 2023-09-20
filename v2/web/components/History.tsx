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
    homeDir: string | null;
    dispatch: Dispatch;
}

function renderHistoryItem(item: HistoryItem): React.ReactNode {
    return item.text;
}

function text(path: string, homeDir: string | null): string {
    if (homeDir && path.startsWith(homeDir)) {
        return `~${path.slice(homeDir.length)}`;
    }
    if (path.startsWith('\\\\?\\')) {
        return path.slice(4); // Strip UNC path
    }
    return path;
}

export const History: React.FC<Props> = ({ history, homeDir, dispatch }) => {
    const items = useMemo(() => {
        const items = history.map(path => ({ text: text(path, homeDir), path }));
        return items.reverse();
    }, [history, homeDir]);

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
