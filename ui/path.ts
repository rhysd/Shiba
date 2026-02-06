export function displayPath(path: string, homeDir: string | null): string {
    if (homeDir && path.startsWith(homeDir)) {
        return `~${path.slice(homeDir.length)}`;
    }
    if (path.startsWith('\\\\?\\')) {
        return path.slice(4); // Strip UNC path
    }
    return path;
}
