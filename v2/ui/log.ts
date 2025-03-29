import { sendMessage } from './ipc';

export let debug: (...args: unknown[]) => void = function nop() {
    // Do not show debug logs by default
};
export function error(...args: unknown[]): void {
    console.error(...args); // eslint-disable-line no-console
    const message = args
        .map(x => {
            if (x?.constructor === Object) {
                return JSON.stringify(x);
            } else {
                return String(x).replace('\n', ' ');
            }
        })
        .join(' ');
    sendMessage({ kind: 'error', message });
}

export function enableDebug(): void {
    debug = console.debug; // eslint-disable-line no-console
}
