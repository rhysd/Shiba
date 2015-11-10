/// <reference path="../typings/tsd.d.ts" />

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

