/// <reference path="../typings/main.d.ts" />
/// <reference path="./config.d.ts" />

declare module NodeJS {
    interface Process {
        resourcesPath: string;
    }
    interface Global {
        config: Config;
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

interface LinterMessage {
    header: string;
    body: string;
}

