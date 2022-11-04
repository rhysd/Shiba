import React, { useState, useEffect } from 'react';
import { Search } from './Search';
import type { Shiba } from '../shiba';

interface Props {
    app: Shiba;
}

export const Preview: React.FC<Props> = ({app}) => {
    const [elem, setElem] = useState<React.ReactNode | null>(null);
    const [searchOpen, setSearchOpen] = useState(false);

    useEffect(() => {
        app.registerPreviewCallback(setElem);
        app.registerStartSearch(() => setSearchOpen(true));
        // Unregistering the callback may be better on unmount
    });

    return <>
        {searchOpen ? <Search app={app} onClose={() => setSearchOpen(false)}/> : undefined}
        <article className="markdown-body">{elem}</article>
    </>;
};
