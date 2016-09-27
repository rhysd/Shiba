declare module 'touch' {
    interface TouchOptions {
        force: boolean;
        time: number;
        atime: boolean | Date;
        mtime: boolean | Date;
        ref: string;
        nocreate: boolean;
    }

    interface TouchModule {
        (filename: string, options?: TouchOptions, cb?: (err: Error) => void): void;
        sync(filename: string, options?: TouchOptions): void;
    }

    const touch: TouchModule;
    export = touch;
}
