import React, { useEffect } from 'react';
import { Search } from './Search';
import type { Dispatch, State } from '../reducer';
import * as log from '../log';

function appearInViewport(elem: Element): boolean {
    const { top, left, bottom, right } = elem.getBoundingClientRect();
    const height = window.innerHeight ?? document.documentElement.clientHeight;
    const width = window.innerWidth ?? document.documentElement.clientWidth;
    const outside = bottom < 0 || height < top || right < 0 || width < left;
    return !outside;
}

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

    useEffect(() => {
        if (search && preview) {
            return;
        }
        const marker = document.querySelector('.last-modified-marker');
        if (marker !== null && !appearInViewport(marker)) {
            log.debug('Scrolling to last modified element:', marker);
            marker.scrollIntoView({
                behavior: 'smooth', // This does not work on WKWebView
                block: 'center',
                inline: 'center',
            });
        }
    });

    return (
        <>
            {searchInput}
            <article className="markdown-body">{preview?.react}</article>
        </>
    );
};
