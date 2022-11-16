import * as log from './log';

function isInViewport(elem: Element): boolean {
    const rect = elem.getBoundingClientRect();
    const height = window.innerHeight ?? document.documentElement.clientHeight;
    const width = window.innerWidth ?? document.documentElement.clientWidth;
    return 0 <= rect.top && 0 <= rect.left && rect.bottom <= height && rect.right <= width;
}

function scrollIntoView(elem: Element): void {
    if (isInViewport(elem)) {
        return;
    }
    log.debug('Scrolling to current match element:', elem);
    elem.scrollIntoView({
        behavior: 'smooth', // This does not work on WKWebView
        block: 'center',
        inline: 'center',
    });
}

export function countSearchMatches(): number {
    return document.querySelectorAll('.search-text-start,.search-text-current-start').length;
}

export function findAllSearchMatchElems(): HTMLElement[] {
    const nodes: NodeListOf<HTMLElement> = document.querySelectorAll(
        '.search-text-start,.search-text-current-start,.search-text,.search-text-current',
    );
    return Array.from(nodes);
}

function updateMatchClassNames(className: string, startIndex: number, elems: HTMLElement[]): void {
    elems[startIndex].className = `${className}-start`;
    for (let i = startIndex + 1; i < elems.length; i++) {
        const e = elems[i];
        if (e.className.endsWith('-start')) {
            break;
        }
        e.className = className;
    }
}

export function searchNextIndex(index: number | null): number | null {
    const all = findAllSearchMatchElems();
    const startIndices: number[] = [];
    for (let i = 0; i < all.length; i++) {
        if (all[i].className.endsWith('-start')) {
            startIndices.push(i);
        }
    }

    let next;
    if (startIndices.length === 0) {
        next = null;
    } else if (index !== null) {
        next = index + 1 >= startIndices.length ? 0 : index + 1;
    } else {
        // Find the nearest next item against current scroll position
        const y = window.scrollY;
        for (const i of startIndices) {
            if (all[i].offsetTop >= y) {
                next = i;
                break;
            }
        }
        next ??= 0;
    }

    log.debug('Search next index:', index, next);
    if (index !== next) {
        if (index !== null) {
            updateMatchClassNames('search-text', startIndices[index], all);
        }
        if (next !== null) {
            const startIndex = startIndices[next];
            updateMatchClassNames('search-text-current', startIndex, all);
            scrollIntoView(all[startIndex]);
        }
    }

    return next;
}

export function searchPreviousIndex(index: number | null): number | null {
    const all = findAllSearchMatchElems();
    const startIndices: number[] = [];
    for (let i = 0; i < all.length; i++) {
        if (all[i].className.endsWith('-start')) {
            startIndices.push(i);
        }
    }

    let next;
    if (startIndices.length === 0) {
        next = null;
    } else if (index !== null) {
        next = index > 0 ? index - 1 : startIndices.length - 1;
    } else {
        // Find the nearest previous item against current scroll position
        const height = window.innerHeight ?? document.documentElement.clientHeight;
        const y = window.scrollY + height;
        for (const i of startIndices) {
            const e = all[i];
            const bottom = e.offsetTop + e.clientHeight;
            if (bottom >= y) {
                next = i - 1;
                break;
            }
        }
        next = next !== undefined && next >= 0 ? next : startIndices.length - 1;
    }

    log.debug('Search previous index:', index, next);
    if (index !== next) {
        if (index !== null) {
            updateMatchClassNames('search-text', startIndices[index], all);
        }
        if (next !== null) {
            const startIndex = startIndices[next];
            updateMatchClassNames('search-text-current', startIndex, all);
            scrollIntoView(all[startIndex]);
        }
    }

    return next;
}
