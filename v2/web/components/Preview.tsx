import React, { useState, useEffect } from 'react';
import { useShiba } from './context';

export const Preview: React.FC = () => {
    const [elem, setElem] = useState<React.ReactNode | null>(null);
    const app = useShiba();

    useEffect(() => {
        app.registerPreviewCallback(setElem);
        // Unregistering the callback may be better on unmount
    });

    return <article className="markdown-body">{elem}</article>;
};
