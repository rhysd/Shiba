export let debug: (...args: unknown[]) => void = function nop() {};
export const error = console.error; // Sending error to the main may be better

export function enableDebug() {
    debug = console.debug;
}
