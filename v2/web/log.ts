import { sendMessage } from './ipc';

export let debug: (...args: unknown[]) => void = function nop() {
    // Do not show debug logs by default
};
export function error(...args: any[]): void {
    console.error(...args);
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
    debug = console.debug;
}
