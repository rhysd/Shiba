import * as React from 'react';
import { useEffect, useRef } from 'react';
import type { MarkdownReactTree } from '../markdown';
import * as log from '../log';

function appearInViewport(elem: Element): boolean {
    const { top, left, bottom, right } = elem.getBoundingClientRect();
    const height = window.innerHeight ?? document.documentElement.clientHeight;
    const width = window.innerWidth ?? document.documentElement.clientWidth;
    const outside = bottom < 0 || height < top || right < 0 || width < left;
    return !outside;
}

export interface Props {
    tree: MarkdownReactTree;
}

export const Preview: React.FC<Props> = ({ tree }) => {
    const { root, lastModified } = tree;
    const ref = useRef<HTMLElement>(null);

    useEffect(() => {
        const elem = lastModified?.current;
        if (!elem || appearInViewport(elem)) {
            return;
        }
        log.debug('Scrolling to last modified element:', elem);
        elem.scrollIntoView({
            behavior: 'smooth', // This does not work on WKWebView
            block: 'center',
            inline: 'center',
        });
    }, [lastModified]);

    useEffect(() => {
        const article = ref.current;
        if (article === null) {
            return;
        }

        const bg = window.getComputedStyle(article, null).getPropertyValue('background-color');
        if (!bg) {
            return;
        }

        document.documentElement.style.backgroundColor = bg;
    }, []);

    let style;
    if (root === null) {
        style = { display: 'none' };
    }

    return (
        <article className="markdown-body" style={style} ref={ref}>
            {root}
        </article>
    );
};
