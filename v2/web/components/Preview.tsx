import React from 'react';
import { Search } from './Search';
import type { Dispatch, State } from '../reducer';

interface Props {
    dispatch: Dispatch;
    state: State;
}

export const Preview: React.FC<Props> = ({ state, dispatch }) => {
    const { preview, search, matcher } = state;

    let searchInput;
    if (search && preview) {
        searchInput = (
            <Search previewContent={preview.hast} index={search.index} matcher={matcher} dispatch={dispatch} />
        );
    }

    return (
        <>
            {searchInput}
            <article className="markdown-body">{preview?.react}</article>
        </>
    );
};
