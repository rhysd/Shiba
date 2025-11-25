import { sendMessage } from './ipc';
import * as log from './log';

type Listener = (isDark: boolean) => void;

class ColorScheme {
    private readonly listeners: Listener[] = [];
    private dark: boolean;

    constructor() {
        const media = window.matchMedia('(prefers-color-scheme: dark)');
        media.addEventListener('change', this.onEvent.bind(this));
        this.dark = media.matches;
    }

    get isDark(): boolean {
        return this.dark;
    }

    addListener(listener: Listener): void {
        this.listeners.push(listener);
    }

    private onEvent(event: MediaQueryListEvent): void {
        if (this.dark === event.matches) {
            return;
        }
        this.dark = !this.dark;
        for (const listener of this.listeners) {
            listener(this.dark);
        }
        sendMessage({ kind: 'reload' }); // Rerender the preview with the changed theme after all callbacks were run
        log.debug('prefers-color-scheme media query has changed. isDark:', !this.dark, '->', this.dark);
    }
}

export const colorScheme = new ColorScheme();

export function parseColor(color: string): [number, number, number] | null {
    if (!color.startsWith('#')) {
        return null;
    }
    switch (color.length) {
        case 7: {
            const r = parseInt(color.slice(1, 3), 16);
            const g = parseInt(color.slice(3, 5), 16);
            const b = parseInt(color.slice(5, 7), 16);
            return [r, g, b];
        }
        case 4: {
            const r = parseInt(color.charAt(1), 16) * 0x11;
            const g = parseInt(color.charAt(2), 16) * 0x11;
            const b = parseInt(color.charAt(3), 16) * 0x11;
            return [r, g, b];
        }
        default:
            return null;
    }
}
