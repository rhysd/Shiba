import * as React from 'react';
import { useEffect, useRef } from 'react';
import type { MarkdownReactTree } from '../markdown';
import { type Dispatch, type Heading, updateHeadings } from '../reducer';
import * as log from '../log';

function appearInViewport(elem: Element): boolean {
    const { top, left, bottom, right } = elem.getBoundingClientRect();
    const height = window.innerHeight;
    const width = window.innerWidth;
    const outside = bottom < 0 || height < top || right < 0 || width < left;
    return !outside;
}

function getTitle(elem: HTMLElement): string {
    const text = elem.textContent;
    if (text) {
        return text;
    }

    type AltElem = (HTMLElement & { alt: string }) | null;
    const alt = (elem.querySelector('*[alt]') as AltElem)?.alt;
    if (alt) {
        return alt;
    }

    return '(empty title)';
}

function collectHeadings(root: HTMLElement): Heading[] {
    const headings: Heading[] = [];
    for (const elem of root.querySelectorAll<HTMLHeadingElement>('article > h1,h2,h3,h4,h5,h6')) {
        const level = parseInt(elem.tagName.slice(1), 10);
        const text = getTitle(elem);
        headings.push({ level, text, elem });
    }

    const scrollTop = root.scrollTop;
    const scrollBottom = scrollTop + root.clientHeight;
    let found = false;
    for (let i = 0; i < headings.length; i++) {
        const top = headings[i].elem.offsetTop;
        if (top >= scrollTop) {
            const j = top >= scrollBottom && i > 0 ? i - 1 : i;
            headings[j].current = true;
            found = true;
            break;
        }
    }

    // Current scroll position is below the last heading.
    if (!found && headings.length > 0) {
        headings[headings.length - 1].current = true;
    }

    return headings;
}

let currentId: number | null = null;
function dispatchHeadings(root: HTMLElement, dispatch: Dispatch): void {
    if (currentId !== null) {
        clearTimeout(currentId);
        currentId = null;
    }

    const callback = (): void => {
        dispatch(updateHeadings(collectHeadings(root)));
        currentId = null;
    };

    currentId = setTimeout(callback, 100);
}

export interface Props {
    tree: MarkdownReactTree;
    dispatch: Dispatch;
}

export const Article: React.FC<Props> = ({ tree, dispatch }) => {
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
        if (root && ref.current) {
            dispatchHeadings(ref.current, dispatch);
        }
    }, [root, dispatch]);

    useEffect(() => {
        if (ref.current) {
            const elem = ref.current;
            elem.addEventListener(
                'scroll',
                () => {
                    dispatchHeadings(elem, dispatch);
                },
                { passive: true },
            );
        }
    }, [dispatch]);

    return (
        <article className="markdown-body" ref={ref}>
            {root}
        </article>
    );
};
