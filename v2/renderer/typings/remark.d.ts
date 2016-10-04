declare namespace Remark {
    export interface NodeData {
        hProperties?: {
            className?: string[];
            [prop: string]: string | string[];
        };
        htmlAttributes?: {
            [attr: string]: string;
        };
        hChildren?: {
            type: string;
            tagName: string;
            properties: {
                className: string[];
            };
        };
        id?: string;
    }

    export type ReferenceType = 'full' | 'shortcut' | 'collapsed';
    export type TableAlign = null | 'right' | 'left' | 'center';

    export interface Link extends Unist.Parent {
        type: 'link';
        title: string | null;
        url: string;
    }

    export interface Text extends Unist.Text {
        type: 'text';
    }

    export interface Heading extends Unist.Parent {
        type: 'heading';
        depth: number;
        data: NodeData;
    }

    export interface Paragraph extends Unist.Parent {
        type: 'paragraph';
    }

    export interface ListItem extends Unist.Parent {
        type: 'listItem';
        loose: boolean;
        checked: boolean | null;
    }

    export interface List extends Unist.Parent {
        type: 'list';
        ordered: boolean;
        start: number | null;
        loose: boolean;
    }

    export interface Code extends Unist.Parent {
        type: 'code';
        lang: string | null;
        value: string;
    }

    export interface InlineCode extends Unist.Node {
        type: 'inlineCode';
        value: string;
    }

    export interface Image extends Unist.Node {
        type: 'image';
        title: string | null;
        url: string;
        alt: string | null;
    }

    export interface ImageReference extends Unist.Node {
        type: 'imageReference';
        identifier: string;
        referenceType: ReferenceType;
        alt: string | null;
    }

    export interface Definition extends Unist.Node {
        type: 'definition';
        identifier: string;
        title: string | null;
        url: string;
    }

    export interface LinkReference extends Unist.Parent {
        type: 'linkReference';
        identifier: string;
        referenceType: ReferenceType;
    }

    export interface Emphasis extends Unist.Parent {
        type: 'emphasis';
    }

    export interface Strong extends Unist.Parent {
        type: 'strong';
    }

    export interface ThematicBreak extends Unist.Node {
        type: 'thematicBreak';
    }

    export interface Blockquote extends Unist.Parent {
        type: 'blockquote';
    }

    export interface Html extends Unist.Node {
        type: 'html';
        value: string;
    }

    export interface TableCell extends Unist.Parent {
        type: 'tableCell';
    }

    export interface TableRow extends Unist.Parent {
        type: 'tableRow';
    }

    export interface Table extends Unist.Parent {
        type: 'table';
        align: TableAlign[];
    }

    export interface Delete extends Unist.Parent {
        type: 'delete';
    }

    export interface Root extends Unist.Parent {
        type: 'root';
    }

    export type MarkdownNode
        = Link
        | Text
        | Heading
        | Paragraph
        | ListItem
        | List
        | Code
        | InlineCode
        | Image
        | ImageReference
        | Definition
        | LinkReference
        | Emphasis
        | Strong
        | ThematicBreak
        | Blockquote
        | Html
        | TableCell
        | TableRow
        | Table
        | Delete
        | Root
    ;

    export interface CompilerPlugin {
        (processor: Unified.Processor, options: Unified.Options): Unified.Processor;
    }
    export interface TransformerPlugin {
        (processor: Unified.Processor, options: Unified.Options): (tree: Root, file?: Unified.VFile) => void;
    }
    export interface ParserPlugin {
        (processor: Unified.Processor, options: Unified.Options): Unified.Processor;
    }
}

declare module 'remark' {
    function remark(): Unified.Processor;
    export = remark;
}
declare module 'remark-parse' {
    const plugin: Remark.ParserPlugin;
    export = plugin;
}
declare module 'remark-slug' {
    const plugin: Remark.TransformerPlugin;
    export = plugin;
}
declare module 'remark-autolink-headings' {
    const plugin: Remark.TransformerPlugin;
    export = plugin;
}
declare module 'remark-github' {
    const plugin: Remark.TransformerPlugin;
    export = plugin;
}
declare module 'remark-toc' {
    const plugin: Remark.TransformerPlugin;
    export = plugin;
}
declare module 'remark-rehype' {
    const plugin: Remark.TransformerPlugin;
    export = plugin;
}
declare module 'remark-emoji' {
    const plugin: Remark.TransformerPlugin;
    export = plugin;
}
declare module 'remark-react' {
    const plugin: Remark.CompilerPlugin;
    export = plugin;
}
declare module 'remark-lint' {
    const plugin: Remark.CompilerPlugin;
    export = plugin;
}
