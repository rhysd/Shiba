import React from 'react';
import { mathjax } from 'mathjax-full/js/mathjax';
import type { MathDocument } from 'mathjax-full/js/core/MathDocument';
import { TeX } from 'mathjax-full/js/input/tex';
import { SVG } from 'mathjax-full/js/output/svg';
import { AllPackages } from 'mathjax-full/js/input/tex/AllPackages';
import { liteAdaptor, type LiteAdaptor } from 'mathjax-full/js/adaptors/liteAdaptor';
import { RegisterHTMLHandler } from 'mathjax-full/js/handlers/html';
import type { LiteElement } from 'mathjax-full/js/adaptors/lite/Element';
import type { LiteText } from 'mathjax-full/js/adaptors/lite/Text';
import type { LiteDocument } from 'mathjax-full/js/adaptors/lite/Document';
import * as log from '../log';

type Document = MathDocument<LiteElement, LiteText, LiteDocument>;
type Converter = [Document, LiteAdaptor];

let cache: Converter | null = null;
function getConverter(): Converter {
    if (cache !== null) {
        return cache;
    }

    const adaptor = liteAdaptor();
    RegisterHTMLHandler(adaptor);
    const document = mathjax.document('', {
        InputJax: new TeX({ packages: AllPackages }),
        OutputJax: new SVG({ fontCache: 'local' }),
    });
    cache = [document, adaptor];

    log.debug('Initialized Mathjax renderer', cache);
    return cache;
}

export interface Props {
    expr: string;
    className: 'math-expr-block' | 'math-expr-inline' | 'code-fence-math';
}

export const Mathjax: React.FC<Props> = ({ expr, className }) => {
    const [document, adaptor] = getConverter();
    const node = document.convert(expr) as LiteElement;
    const html = adaptor.innerHTML(node);
    return <span className={className} dangerouslySetInnerHTML={{ __html: html }} />; // eslint-disable-line @typescript-eslint/naming-convention
};
