declare namespace Unist {
    export interface Node {
    }

    interface H {
        (node: Remark.MarkdownNode, tag: string, props: Object, children: Unist.Node[]): Unist.Node;
        augment(node: Remark.MarkdownNode, parent: Unist.Node): Unist.Node;
    }

    interface U {
        (type: string, value: string): Unist.Node;
    }
}

declare module 'unist-builder' {
    const builder: Unist.U;
    export = builder;
}

declare module 'unist-util-visit' {
    interface Visitor {
        (node: Remark.MarkdownNode, type: 'link',           cb: (node: Remark.Link) => void): void;
        (node: Remark.MarkdownNode, type: 'text',           cb: (node: Remark.Text) => void): void;
        (node: Remark.MarkdownNode, type: 'heading',        cb: (node: Remark.Heading) => void): void;
        (node: Remark.MarkdownNode, type: 'paragraph',      cb: (node: Remark.Paragraph) => void): void;
        (node: Remark.MarkdownNode, type: 'listItem',       cb: (node: Remark.ListItem) => void): void;
        (node: Remark.MarkdownNode, type: 'list',           cb: (node: Remark.List) => void): void;
        (node: Remark.MarkdownNode, type: 'code',           cb: (node: Remark.Code) => void): void;
        (node: Remark.MarkdownNode, type: 'inlineCode',     cb: (node: Remark.InlineCode) => void): void;
        (node: Remark.MarkdownNode, type: 'image',          cb: (node: Remark.Image) => void): void;
        (node: Remark.MarkdownNode, type: 'imageReference', cb: (node: Remark.ImageReference) => void): void;
        (node: Remark.MarkdownNode, type: 'definition',     cb: (node: Remark.Definition) => void): void;
        (node: Remark.MarkdownNode, type: 'linkReference',  cb: (node: Remark.LinkReference) => void): void;
        (node: Remark.MarkdownNode, type: 'emphasis',       cb: (node: Remark.Emphasis) => void): void;
        (node: Remark.MarkdownNode, type: 'strong',         cb: (node: Remark.Strong) => void): void;
        (node: Remark.MarkdownNode, type: 'thematicBreak',  cb: (node: Remark.ThematicBreak) => void): void;
        (node: Remark.MarkdownNode, type: 'blockquote',     cb: (node: Remark.Blockquote) => void): void;
        (node: Remark.MarkdownNode, type: 'html',           cb: (node: Remark.Html) => void): void;
        (node: Remark.MarkdownNode, type: 'tableCell',      cb: (node: Remark.TableCell) => void): void;
        (node: Remark.MarkdownNode, type: 'tableRow',       cb: (node: Remark.TableRow) => void): void;
        (node: Remark.MarkdownNode, type: 'table',          cb: (node: Remark.Table) => void): void;
        (node: Remark.MarkdownNode, type: 'delete',         cb: (node: Remark.Delete) => void): void;
        (node: Remark.MarkdownNode, type: 'root',           cb: (node: Remark.Root) => void): void;
        (node: Remark.MarkdownNode, type: string,           cb: Function): void;
    }
    const v: Visitor;
    export = v;
}
