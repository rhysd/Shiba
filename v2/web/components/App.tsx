import React, { useReducer, useEffect } from 'react';
import { Search } from './Search';
import { sendMessage } from '../ipc';
import { INITIAL_STATE, reducer } from '../reducer';
import type { Dispatcher } from '../dispatcher';
import * as log from '../log';

function appearInViewport(elem: Element): boolean {
    const { top, left, bottom, right } = elem.getBoundingClientRect();
    const height = window.innerHeight ?? document.documentElement.clientHeight;
    const width = window.innerWidth ?? document.documentElement.clientWidth;
    const outside = bottom < 0 || height < top || right < 0 || width < left;
    return !outside;
}

interface Props {
    dispatcher: Dispatcher;
}

export const App: React.FC<Props> = ({ dispatcher }) => {
    const [state, dispatch] = useReducer(reducer, INITIAL_STATE);
    const { searching, searchIndex, matcher } = state;

    let searchInput;
    if (searching) {
        searchInput = <Search index={searchIndex} matcher={matcher} dispatch={dispatch} />;
    }

    useEffect(() => {
        dispatcher.setDispatch(dispatch, state);

        if (!searching) {
            const marker = document.querySelector('.last-modified-marker');
            if (marker !== null && !appearInViewport(marker)) {
                log.debug('Scrolling to last modified element:', marker);
                marker.scrollIntoView({
                    behavior: 'smooth', // This does not work on WKWebView
                    block: 'center',
                    inline: 'center',
                });
            }
        }
    });
    useEffect(() => {
        sendMessage({ kind: 'init' });
    }, []); // Run only when component was mounted

    return <>{searchInput}</>;
};
