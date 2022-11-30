import React from 'react';
import { mathjax } from 'mathjax-full/js/mathjax';
import type { MathDocument } from 'mathjax-full/js/core/MathDocument';
import { TeX } from 'mathjax-full/js/input/tex';
import { SVG } from 'mathjax-full/js/output/svg';
import { AllPackages } from 'mathjax-full/js/input/tex/AllPackages';
import { liteAdaptor } from 'mathjax-full/js/adaptors/liteAdaptor';
import { RegisterHTMLHandler } from 'mathjax-full/js/handlers/html';
import * as log from '../log';

type Document = MathDocument<any, any, any>;
const ADAPTOR = liteAdaptor();

let documentCache: Document | null = null;
function getDocument(): Document {
    if (documentCache !== null) {
        return documentCache;
    }

    RegisterHTMLHandler(ADAPTOR);
    documentCache = mathjax.document('', {
        InputJax: new TeX({ packages: AllPackages }),
        OutputJax: new SVG({ fontCache: 'local' }),
    });

    log.debug('Initialized Mathjax renderer', documentCache);
    return documentCache;
}

export interface Props {
    expr: string;
    className: 'math-expr-block' | 'math-expr-inline' | 'code-fence-math';
}

export const Mathjax: React.FC<Props> = ({ expr, className }) => {
    const document = getDocument();
    const node = document.convert(expr);
    const html = ADAPTOR.innerHTML(node);
    return <span className={className} dangerouslySetInnerHTML={{ __html: html }} />; // eslint-disable-line @typescript-eslint/naming-convention
};
