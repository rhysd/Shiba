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
