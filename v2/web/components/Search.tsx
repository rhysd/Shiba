import React from 'react';
import * as log from '../log';
import { Dispatch, searchText, closeSearch } from '../reducer';
import type { Root as Hast } from 'hast';

interface Props {
    previewContent: Hast,
    dispatch: Dispatch;
}

export const Search: React.FC<Props> = ({previewContent, dispatch}) => {
    const prev = (e: React.MouseEvent) => {
        e.preventDefault();
        log.debug('TODO: Search previous');
    };
    const next = (e: React.MouseEvent) => {
        e.preventDefault();
        log.debug('TODO: Search next');
    };
    const close = (e: React.MouseEvent) => {
        e.preventDefault();
        dispatch(closeSearch());
    };
    const onChange = async (e: React.FormEvent<HTMLInputElement>) => {
        dispatch(await searchText(previewContent, e.currentTarget.value));
    };
    return <div className="search-text-box">
        <span className="search-text-icon">🔍</span>
        <input className="search-text-input" onChange={onChange} type="search" placeholder="Search…" autoFocus />
        <span className="search-text-button" onClick={prev}>&lt;</span>
        <span className="search-text-button" onClick={next}>&gt;</span>
        <span className="search-text-button" onClick={close}>×</span>
    </div>
};
