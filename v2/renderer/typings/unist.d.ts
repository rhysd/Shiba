declare namespace Unist {
    export interface Location {
        line: number;
        column: number;
        offset: number;
    }

    export interface Position {
        start: Location;
        end: Location;
        indent: number[];
    }

    export interface Node {
        type: string;
        data?: Object;
        position?: Position;
    }

    export interface Text extends Node {
        value: string;
    }

    export interface Parent extends Node {
        children: this[];
    }

    interface H {
        (node: Remark.MarkdownNode, tag: string, props: Object, children: Unist.Node[]): Unist.Node;
        augment(node: Remark.MarkdownNode, parent: Unist.Node): Unist.Node;
    }

    interface U {
        (type: 'text', value: string): Unist.Text;
        (type: 'element', props: Object, children: Unist.Node[]): Unist.Node;
        (type: string, ...args: any[]): Unist.Node;
    }
}

declare module 'unist-builder' {
    const builder: Unist.U;
    export = builder;
}

declare module 'unist-util-visit' {
    interface Visitor {
        (node: Unist.Node, type: string, cb: (node: Unist.Node) => void): void;
    }
    const v: Visitor;
    export = v;
}

declare namespace Unified {
    export interface Settings {
        [key: string]: any;
    }
    export type Plugin
        = Remark.CompilerPlugin | Remark.TransformerPlugin | Remark.ParserPlugin
        | Hast.CompilerPlugin | Hast.TransformerPlugin | Hast.ParserPlugin
    ;
    interface PluginOptions {
        [prop: string]: string;
    }
    interface Parser {
        parse(processor: Processor): Unist.Node;
    }
    interface Compiler {
        compile(root: Unist.Node): any;
    }
    type PluginsConfig = {[name: string]: PluginOptions | null} | string[] | string;
    type PresetsConfig = PluginsConfig;
    export type ConfigTransform = (options: Options) => Options;
    export interface Options {
        processor?: Processor;
        cwd?: string;
        globs?: string[];
        extensions?: string[];
        streamIn?: NodeJS.ReadableStream;
        filePath?: string;
        streamOut?: NodeJS.WritableStream;
        streamError?: NodeJS.WritableStream;
        out?: boolean;
        output?: string | boolean;
        tree?: boolean;
        treeIn?: boolean;
        treeOut?: boolean;
        rcName?: string;
        packageField?: string;
        detectConfig?: boolean;
        rcPath?: string;
        settings?: Settings;
        ignoreName?: string;
        detectIgnore?: boolean;
        ignorePath?: string;
        silentlyIgnore?: boolean;
        plugins?: PluginsConfig;
        presets?: PresetsConfig;
        pluginPrefix?: string;
        presetPrefix?: string | boolean;
        configTransform?: ConfigTransform;
        injectedPlugins?: Plugin[];
        color?: boolean;
        silent?: boolean;
        quiet?: boolean;
        frail?: boolean;
        files?: VFile[];
        [name: string]: any;
    }
    export interface VMessage {
        message: string;
        name: string;
        file: string;
        reason: string;
        line: number;
        column: number;
        location: Unist.Position;
        ruleId: string | null;
        source: string | null; // Plugin name
        fatal: boolean;
    }
    export class VFile {
        data: Object;
        messages: VMessage[];
        history: any[];
        cwd: string;
        contents: any;
    }
    export type ProcessCallback = (err: Error, result: VFile) => void;
    export class Processor {
        Parser: Parser;
        Compiler: Compiler;
        use(plugin: Plugin | Plugin[], settings?: Settings): Processor;
        run(node: Unist.Node, file?: VFile, done?: (err: Error, node: Unist.Node, file: VFile) => void): Unist.Node;
        process(source: string | string[], cb: ProcessCallback): void;
        parse(file: VFile, options?: Options): Unist.Node;
        stringify(node: Unist.Node, file?: VFile, options?: Options): string;
        write(chunk: Buffer | string, encoding?: string, callback?: Function): boolean;
        pipe(stream: NodeJS.WritableStream): NodeJS.WritableStream;
        data(key: string, value?: any): any;
        end(): boolean;
        abstract(): Processor;
    }
}

declare module 'unified' {
    const engine: (options?: Unified.Options) => Unified.Processor;
    export = engine;
}

