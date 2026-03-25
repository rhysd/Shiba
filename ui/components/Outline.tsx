import * as React from 'react';
import { useMemo, useCallback } from 'react';
import { Palette } from './Palette';
import { type Dispatch, closeOutline } from '../reducer';
import { sendMessage } from '../ipc';

interface Heading {
    prefix: string;
    text: string;
    index: number;
    elem: HTMLHeadingElement;
}

function collectHeadings(): Heading[] {
    const ret = [];
    const elems: NodeListOf<HTMLHeadingElement> = document.querySelectorAll('article > h1,h2,h3,h4,h5,h6');
    let index = 0;
    for (const elem of elems) {
        const prefix = '#'.repeat(parseInt(elem.tagName.slice(1), 10));
        const text = elem.textContent;
        ret.push({ prefix, text, elem, index });
        index++;
    }
    return ret;
}

function renderHeading(h: Heading): React.ReactNode {
    return `${h.prefix} ${h.text}`;
}

interface Props {
    dispatch: Dispatch;
}

export const Outline: React.FC<Props> = ({ dispatch }) => {
    const headings = useMemo(() => collectHeadings(), []);

    const handleClose = useCallback((): void => {
        dispatch(closeOutline());
    }, [dispatch]);

    const handleSelect = useCallback(
        (heading: Heading, shiftKey: boolean): void => {
            if (shiftKey) {
                sendMessage({ kind: 'duplicate_window', heading: heading.index });
            } else {
                heading.elem.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start',
                    inline: 'start',
                });
            }
            dispatch(closeOutline());
        },
        [dispatch],
    );

    return (
        <Palette
            items={headings}
            placeholder="Search outline…"
            onClose={handleClose}
            onSelect={handleSelect}
            renderItem={renderHeading}
        />
    );
};
