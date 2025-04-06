import * as React from 'react';
import { useEffect, useRef } from 'react';

export interface Props {
    svg: string;
    bindFn: ((elem: Element) => void) | undefined;
}

export const Mermaid: React.FC<Props> = ({ svg, bindFn }) => {
    const ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (ref.current !== null && bindFn) {
            bindFn(ref.current);
        }
    }, [bindFn]);

    return <div className="mermaid" dangerouslySetInnerHTML={{ __html: svg }} ref={ref} />; // eslint-disable-line @typescript-eslint/naming-convention
};
