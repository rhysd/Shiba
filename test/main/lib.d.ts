/// <reference path="../../browser/lib.d.ts" />

declare module 'touch' {
    interface TouchOptions {
        force: boolean;
        time: number;
        atime: boolean | Date;
        mtime: boolean | Date;
        ref: string;
        nocreate: boolean;
    }

    const touch: (filename: string, options?: TouchOptions, cb?: (err: Error) => void) => void;
    export = touch;
}
