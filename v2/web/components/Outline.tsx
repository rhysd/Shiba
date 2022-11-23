import React, { useMemo } from 'react';
import { Palette } from './Palette';
import { type Dispatch, closeOutline } from '../reducer';

interface Heading {
    prefix: string;
    text: string;
    elem: HTMLHeadingElement;
}

function collectHeadings(): Heading[] {
    const ret = [];
    for (const elem of document.querySelectorAll('h1,h2,h3,h4,h5,h6') as NodeListOf<HTMLHeadingElement>) {
        const prefix = '#'.repeat(parseInt(elem.tagName.slice(1), 10));
        const text = elem.textContent ?? '';
        ret.push({ prefix, text, elem });
    }
    return ret;
}

interface Props {
    dispatch: Dispatch;
}

export const Outline: React.FC<Props> = ({ dispatch }) => {
    const headings = useMemo(collectHeadings, []);

    const handleClose = (): void => {
        dispatch(closeOutline());
    };

    const handleSelect = (heading: Heading): void => {
        heading.elem.scrollIntoView({
            behavior: 'smooth',
            block: 'start',
            inline: 'start',
        });
        dispatch(closeOutline());
    };

    return <Palette items={headings} placeholder="Search outline…" onClose={handleClose} onSelect={handleSelect} />;
};
