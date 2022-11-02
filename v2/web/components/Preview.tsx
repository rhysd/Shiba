import React, { useState, useEffect } from 'react';

export function Preview() {
    const [elem, setElem] = useState(null);
    useEffect(() => {
        window.ShibaApp.registerContentCallback(setElem);
    });

    return <article className="markdown-body">
        {elem}
    </article>;
}
