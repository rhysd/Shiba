import React, { useEffect, useRef } from 'react';
import type { Root as Hast } from 'hast';
import {
    Dispatch,
    SearchState,
    searchQuery,
    searchNext,
    searchPrevious,
    closeSearch,
    findSearchMatchElems,
} from '../reducer';
import * as log from '../log';

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
    const { query, index } = state;
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
            const total = findSearchMatchElems().length;
            counterElem.current.textContent = `${nth} / ${total}`;
        }
    }, [state, previewContent]);

    const handlePrev = (): void => {
        searchPrevious(index, previewContent, query).then(dispatch).catch(log.error);
    };
    const handleNext = (): void => {
        searchNext(index, previewContent, query).then(dispatch).catch(log.error);
    };
    const handleClose = (): void => {
        closeSearch(previewContent).then(dispatch).catch(log.error);
    };
    const handleChange = (e: React.FormEvent<HTMLInputElement>): void => {
        searchQuery(previewContent, e.currentTarget.value, index).then(dispatch).catch(log.error);
    };
    const handleKeydown = (e: React.KeyboardEvent<HTMLInputElement>): void => {
        if (e.key === 'Enter' && !e.shiftKey) {
            handleNext();
        } else if (e.key === 'Enter' && e.shiftKey) {
            handlePrev();
        } else if (e.key === 'Escape') {
            handleClose();
        } else {
            return;
        }
        e.preventDefault();
    };

    return (
        <div className="search-text-box">
            <span className="search-text-icon">🔍</span>
            <input
                className="search-text-input"
                onChange={handleChange}
                onKeyDown={handleKeydown}
                type="search"
                placeholder="Search…"
                autoFocus
            />
            <span className="search-text-counter" ref={counterElem}></span>
            <span className="search-text-button" onClick={handlePrev}>
                &lt;
            </span>
            <span className="search-text-button" onClick={handleNext}>
                &gt;
            </span>
            <span className="search-text-button" onClick={handleClose}>
                ×
            </span>
        </div>
    );
};
