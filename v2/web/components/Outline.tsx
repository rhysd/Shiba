import * as React from 'react';
import { useMemo, useCallback } from 'react';
import { Palette } from './Palette';
import { type Dispatch, closeOutline } from '../reducer';

interface Heading {
    prefix: string;
    text: string;
    elem: HTMLHeadingElement;
}

function collectHeadings(): Heading[] {
    const ret = [];
    for (const elem of document.querySelectorAll('article > h1,h2,h3,h4,h5,h6') as NodeListOf<HTMLHeadingElement>) {
        const prefix = '#'.repeat(parseInt(elem.tagName.slice(1), 10));
        const text = elem.textContent ?? '';
        ret.push({ prefix, text, elem });
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
    const headings = useMemo(collectHeadings, []);

    const handleClose = useCallback((): void => {
        dispatch(closeOutline());
    }, [dispatch]);

    const handleSelect = useCallback(
        (heading: Heading): void => {
            heading.elem.scrollIntoView({
                behavior: 'smooth',
                block: 'start',
                inline: 'start',
            });
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
