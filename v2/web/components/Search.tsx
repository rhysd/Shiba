import React, { useEffect, useRef } from 'react';
import type { Root as Hast } from 'hast';
import {
    Dispatch,
    SearchState,
    searchText,
    searchNext,
    searchPrevious,
    closeSearch,
    countSearchMatches,
} from '../reducer';

function isInViewport(elem: Element): boolean {
    const rect = elem.getBoundingClientRect();
    const height = window.innerHeight ?? document.documentElement.clientHeight;
    const width = window.innerWidth ?? document.documentElement.clientWidth;
    return 0 <= rect.top && 0 <= rect.left && rect.bottom <= height && rect.right <= width;
}

interface Props {
    previewContent: Hast;
    state: SearchState;
    dispatch: Dispatch;
}

export const Search: React.FC<Props> = ({ previewContent, state, dispatch }) => {
    const { text, index } = state;
    const counterElem = useRef<HTMLElement>(null);

    useEffect(() => {
        const current = document.querySelector('.search-text-current');
        if (current && !isInViewport(current)) {
            current.scrollIntoView({
                block: 'center',
                inline: 'center',
            });
        }
        if (counterElem.current !== null) {
            const nth = index !== null ? index + 1 : 0;
            const total = countSearchMatches();
            counterElem.current.textContent = `${nth} / ${total}`;
        }
    }, [state, previewContent]);

    const prev = async () => {
        dispatch(await searchPrevious(index, previewContent, text));
    };
    const next = async () => {
        dispatch(await searchNext(index, previewContent, text));
    };
    const close = async () => {
        dispatch(await closeSearch(previewContent));
    };
    const onChange = async (e: React.FormEvent<HTMLInputElement>) => {
        dispatch(await searchText(previewContent, e.currentTarget.value, index));
    };
    return (
        <div className="search-text-box">
            <span className="search-text-icon">🔍</span>
            <input className="search-text-input" onChange={onChange} type="search" placeholder="Search…" autoFocus />
            <span className="search-text-counter" ref={counterElem}></span>
            <span className="search-text-button" onClick={prev}>
                &lt;
            </span>
            <span className="search-text-button" onClick={next}>
                &gt;
            </span>
            <span className="search-text-button" onClick={close}>
                ×
            </span>
        </div>
    );
};
