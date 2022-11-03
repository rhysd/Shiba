import React, { useState, useEffect } from 'react';
import type { Shiba } from '../shiba';

interface Props {
    app: Shiba;
}

export const Preview: React.FC<Props> = props => {
    const [elem, setElem] = useState<React.ReactNode | null>(null);
    const app = props.app;

    useEffect(() => {
        app.registerPreviewCallback(setElem);
        // Unregistering the callback may be better on unmount
    });

    return <article className="markdown-body">{elem}</article>;
};
