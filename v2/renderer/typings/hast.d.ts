declare namespace Hast {
    export interface Root extends Unist.Parent {
        type: 'root';
    }

    export interface Element extends Unist.Parent {
        type: 'element';
        tagName: string;
        properties: Object;
    }

    export interface Doctype extends Unist.Node {
        type: 'doctype';
        name: string;
        public: string | null;
        system: string | null;
    }

    export interface Comment extends Unist.Text {
        type: 'comment';
    }

    export interface TextNode extends Unist.Text {
        type: 'text';
    }

    type HastNode
        = Root
        | Element
        | Doctype
        | Comment
        | TextNode
    ;

    export interface CompilerPlugin {
        (processor: Unified.Processor, options: Unified.Options): void;
    }
    export interface TransformerPlugin {
        (processor: Unified.Processor, options: Unified.Options): (tree: Root, file?: Unified.VFile) => void;
    }
    export interface ParserPlugin {
        (processor: Unified.Processor, options: Unified.Options): Unified.Processor;
    }
}

declare module 'rehype' {
    const plugin: Hast.ParserPlugin;
    export = plugin;
}

declare module 'rehype-parse' {
    const plugin: Hast.ParserPlugin;
    export = plugin;
}

declare module 'rehype-react' {
    const plugin: Hast.CompilerPlugin;
    export = plugin;
}
