import React, { useState, useEffect } from 'react';

export const Preview: React.FC = () => {
    const [elem, setElem] = useState<React.ReactNode | null>(null);
    useEffect(() => {
        window.ShibaApp.registerContentCallback(setElem);
        // Unregistering the callback may be better on unmount
    });

    return (
        <article className="markdown-body">
            {elem}
        </article>
    );
};
