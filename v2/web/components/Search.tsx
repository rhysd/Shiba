import React, { useEffect, useRef } from 'react';
import type { Root as Hast } from 'hast';
import { Dispatch, SearchState, searchText, closeSearch } from '../reducer';

function isInViewport(elem: Element): boolean {
    const rect = elem.getBoundingClientRect();
    const height = window.innerHeight ?? document.documentElement.clientHeight;
    const width = window.innerWidth ?? document.documentElement.clientWidth;
    return 0 <= rect.top && 0 <= rect.left && rect.bottom <= height && rect.right <= width;
}

function countMatches(): number | null {
    if (!document.querySelector('.search-text-current')) {
        return null;
    }
    return document.querySelectorAll('.search-text').length + 1;
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
        const elem = document.querySelector('.search-text-current');
        if (!elem) {
            if (counterElem.current !== null) {
                counterElem.current.textContent = '0 / 0';
            }
            return;
        }
        if (!isInViewport(elem)) {
            elem.scrollIntoView({
                block: 'nearest',
                inline: 'nearest',
            });
        }
        const total = document.querySelectorAll('.search-text').length + 1;
        if (counterElem.current !== null) {
            counterElem.current.textContent = `${index + 1} / ${total}`;
        }
    }, [state, previewContent]);

    const prev = async () => {
        const count = countMatches();
        if (count !== null) {
            const next = index > 0 ? index - 1 : count - 1;
            dispatch(await searchText(previewContent, text, next));
        }
    };
    const next = async () => {
        const count = countMatches();
        if (count !== null) {
            const next = index + 1 >= count ? 0 : index + 1;
            dispatch(await searchText(previewContent, text, next));
        }
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
