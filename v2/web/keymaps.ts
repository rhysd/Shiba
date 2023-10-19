import { bind as bindKey } from 'mousetrap';
import { openOutline, openHistory, openHelp } from './reducer';
import type { GlobalDispatcher } from './dispatcher';
import { sendMessage, type KeyMaps, type KeyAction } from './ipc';
import * as log from './log';

function scrollTo(candidates: HTMLElement[] | NodeListOf<HTMLElement>, pred: (e: number, s: number) => boolean): void {
    if (candidates.length === 0) {
        return;
    }
    if (document.querySelector('.MuiDialogContent-root')) {
        return; // Do nothing when some dialog is rendered
    }

    const article = document.querySelector('article');
    if (!article) {
        return;
    }

    let scrolled = false;
    const scrollTop = article.scrollTop;
    for (const elem of candidates) {
        const top = elem.offsetTop;
        if (pred(top, scrollTop)) {
            article.scrollTo(0, top);
            if (scrollTop !== article.scrollTop) {
                scrolled = true;
                break;
            }
        }
    }

    if (!scrolled) {
        article.scrollTo(0, candidates[0].clientTop);
    }
}

function scrollTarget(): HTMLElement {
    return (
        document.querySelector('.MuiDialogContent-root') ??
        document.querySelector('article') ??
        document.documentElement
    );
}

export interface KeyShortcut {
    dispatch(dispatcher: GlobalDispatcher): void;
    description: string;
}

const KeyShortcuts: { [K in KeyAction]: KeyShortcut } = {
    ScrollDown: {
        description: 'Scroll down the page by half of window height.',
        dispatch(): void {
            const t = scrollTarget();
            t.scrollBy(0, t.clientHeight / 2);
        },
    },

    ScrollUp: {
        description: 'Scroll up the page by half of window height.',
        dispatch(): void {
            const t = scrollTarget();
            t.scrollBy(0, -t.clientHeight / 2);
        },
    },

    ScrollLeft: {
        description: 'Scroll the page to the left by half of window width.',
        dispatch(): void {
            const t = scrollTarget();
            t.scrollBy(-t.clientWidth / 2, 0);
        },
    },

    ScrollRight: {
        description: 'Scroll the page to the right by half of window width.',
        dispatch(): void {
            const t = scrollTarget();
            t.scrollBy(t.clientWidth / 2, 0);
        },
    },

    ScrollPageDown: {
        description: 'Scroll down the page by window height.',
        dispatch(): void {
            const t = scrollTarget();
            t.scrollBy(0, t.clientHeight);
        },
    },

    ScrollPageUp: {
        description: 'Scroll up the page by window height.',
        dispatch(): void {
            const t = scrollTarget();
            t.scrollBy(0, -t.clientHeight);
        },
    },

    ScrollTop: {
        description: 'Scroll to the top of the page.',
        dispatch(): void {
            scrollTarget().scrollTo(0, 0);
        },
    },

    ScrollBottom: {
        description: 'Scroll to the bottom of the page.',
        dispatch(): void {
            const t = scrollTarget();
            t.scrollTo(0, t.scrollHeight);
        },
    },

    ScrollNextSection: {
        description: 'Scroll to the next section header.',
        dispatch(): void {
            const headings: NodeListOf<HTMLElement> = document.querySelectorAll('article > h1,h2,h3,h4,h5,h6');
            scrollTo(headings, (e, s) => e > s);
        },
    },

    ScrollPrevSection: {
        description: 'Scroll to the previous section header.',
        dispatch(): void {
            const headings: HTMLElement[] = Array.from(document.querySelectorAll('article > h1,h2,h3,h4,h5,h6'));
            headings.reverse();
            scrollTo(headings, (e, s) => e < s);
        },
    },

    Forward: {
        description: 'Go forawrd to the next document in preview history.',
        dispatch(): void {
            sendMessage({ kind: 'forward' });
        },
    },

    Back: {
        description: 'Go backward to the previous document in preview history.',
        dispatch(): void {
            sendMessage({ kind: 'back' });
        },
    },

    Reload: {
        description: 'Reload the current document preview.',
        dispatch(): void {
            sendMessage({ kind: 'reload' });
        },
    },

    OpenFile: {
        description: 'Open a dialog to choose a file to preview.',
        dispatch(): void {
            sendMessage({ kind: 'file_dialog' });
        },
    },

    OpenDir: {
        description: 'Open a dialog to choose a directory to watch file changes.',
        dispatch(): void {
            sendMessage({ kind: 'dir_dialog' });
        },
    },

    Search: {
        description: 'Open an in-page search box.',
        dispatch(dispatcher: GlobalDispatcher): void {
            dispatcher.openSearch();
        },
    },

    SearchNext: {
        description: 'Focus the next match of ongoing text search.',
        dispatch(dispatcher: GlobalDispatcher): void {
            dispatcher.searchNext();
        },
    },

    SearchPrev: {
        description: 'Focus the previous match of the ongoing text search.',
        dispatch(dispatcher: GlobalDispatcher): void {
            dispatcher.searchPrev();
        },
    },

    Outline: {
        description: 'Open a palette to incrementally search the section outline.',
        dispatch(dispatcher: GlobalDispatcher): void {
            dispatcher.dispatch(openOutline());
        },
    },

    History: {
        description: 'Open a palette to incrementally search files in history.',
        dispatch(dispatcher: GlobalDispatcher): void {
            dispatcher.dispatch(openHistory());
        },
    },

    Help: {
        description: 'Show this help.',
        dispatch(dispatcher: GlobalDispatcher): void {
            dispatcher.dispatch(openHelp());
        },
    },

    ZoomIn: {
        description: 'Zoom in on the page.',
        dispatch(): void {
            sendMessage({ kind: 'zoom', zoom: 'In' });
        },
    },

    ZoomOut: {
        description: 'Zoom out on the page.',
        dispatch(): void {
            sendMessage({ kind: 'zoom', zoom: 'Out' });
        },
    },

    ShowMenu: {
        description: 'Show the application menu.',
        dispatch(): void {
            const button = document.getElementById('shiba-menu-button');
            if (button) {
                button.click();
            } else {
                sendMessage({ kind: 'open_menu' }); // Fallback
            }
        },
    },

    ToggleMenuBar: {
        description: 'Toggle menu bar at the top of window.',
        dispatch(): void {
            sendMessage({ kind: 'toggle_menu_bar' });
        },
    },

    Quit: {
        description: 'Quit the application.',
        dispatch(): void {
            sendMessage({ kind: 'quit' });
        },
    },
};

export interface BoundShortcut extends KeyShortcut {
    action: KeyAction;
    binds: string[];
}

export class KeyMapping {
    private sortedShortcuts: BoundShortcut[] = [];

    get shortcuts(): BoundShortcut[] {
        return this.sortedShortcuts;
    }

    register(maps: KeyMaps, dispatcher: GlobalDispatcher): void {
        const bounds = new Map<KeyAction, BoundShortcut>();

        for (const keybind of Object.keys(maps)) {
            const action: KeyAction = maps[keybind];
            if (!(action in KeyShortcuts)) {
                log.error('Unknown key action in keymaps:', keybind, action);
                continue;
            }

            const shortcut = KeyShortcuts[action];
            const method = shortcut.dispatch.bind(undefined, dispatcher);
            bindKey(keybind, event => {
                event.preventDefault();
                event.stopPropagation();
                log.debug('Triggered key shortcut:', action, keybind);
                try {
                    method();
                } catch (err) {
                    log.error('Error while handling key action', action, err);
                }
            });

            const s = bounds.get(action);
            if (s === undefined) {
                bounds.set(action, {
                    ...shortcut,
                    action,
                    binds: [keybind],
                });
            } else {
                s.binds.push(keybind);
            }
        }

        const shortcuts = [...bounds.values()];
        shortcuts.sort((l, r) => l.action.localeCompare(r.action));
        this.sortedShortcuts = shortcuts;
    }
}
