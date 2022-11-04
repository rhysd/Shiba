import React from 'react';
import type { Shiba } from '../shiba';
import * as log from '../log';

interface Props {
    app: Shiba;
    onClose: () => void;
}

export const Search: React.FC<Props> = ({app, onClose}) => {
    const prev = (e: React.MouseEvent) => {
        e.preventDefault();
        log.debug('TODO: Search previous');
    };
    const next = (e: React.MouseEvent) => {
        e.preventDefault();
        log.debug('TODO: Search next');
    };
    const close = async (e: React.MouseEvent) => {
        e.preventDefault();
        await app.search('');
        onClose();
    };
    const onChange = async (e: React.FormEvent<HTMLInputElement>) => {
        await app.search(e.currentTarget.value);
    };
    return <div className="search-text-box">
        <span className="search-text-icon">🔍</span>
        <input className="search-text-input" onChange={onChange} type="search" placeholder="Search…" autoFocus />
        <span className="search-text-button" onClick={prev}>&lt;</span>
        <span className="search-text-button" onClick={next}>&gt;</span>
        <span className="search-text-button" onClick={close}>×</span>
    </div>
};
