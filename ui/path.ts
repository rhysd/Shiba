export function displayPath(path: string, homeDir: string | null): string {
    if (homeDir && path.startsWith(homeDir)) {
        return `~${path.slice(homeDir.length)}`;
    }
    if (path.startsWith('\\\\?\\')) {
        return path.slice(4); // Strip UNC path
    }
    return path;
}

export function fileName(path: string | null): string {
    if (path === null) {
        return '';
    }
    for (const sep of ['/', '\\']) {
        const i = path.lastIndexOf(sep);
        if (i >= 0) {
            return path.slice(i + 1);
        }
    }
    return path;
}
