/// <reference path="../typings/main.d.ts" />

interface Set<T> {
    has(value: T): boolean;
}

interface SetConstructor {
    new <T>(): Set<T>;
    new <T>(iterable: any[]): Set<T>;
    prototype: Set<any>;
}
declare var Set: SetConstructor;

declare module NodeJS {
    export interface Process {
        resourcesPath: string;
    }
}

interface String {
    endsWith(s: string): boolean;
}

declare module 'markdownlint' {
    const lint: (opts: Object, callback: (err: Error, result: any) => void) => void;
    export = lint;
}

declare namespace RemarkLint {
    interface Linter { /* TODO */ }
}

declare module 'remark-lint' {
    const remarklint: RemarkLint.Linter;
    export = remarklint;
}

declare namespace Remark {
    interface Processor {
        use(plugin: RemarkLint.Linter, opts: Object): Processor;
        process(content: string, callback: (err: Error, file: any) => void): void;
    }
}

declare module 'remark' {
    const remark: () => Remark.Processor;
    export = remark;
}
