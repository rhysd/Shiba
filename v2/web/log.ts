import { sendMessage } from './ipc';

export let debug: (...args: unknown[]) => void = function nop() {};
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

export function enableDebug() {
    debug = console.debug;
}
