function compareMessageByStartLocation(l: Unified.VMessage, r: Unified.VMessage): number {
    return compareLocation(l.location.start, r.location.start);
}

function compareLocation(l: Unist.Location, r: Unist.Location): number {
    if (l.line !== r.line) {
        return l.line - r.line;
    }
    return l.column - r.column;
}

// Note: Returns one of
//         - ['text', '<span>text</span>']
//         - ['<span>text</span>']
//         - ['<span>text</span>', 'text']
function getMarkedParts(
    node: Hast.TextNode,
    msg: Unified.VMessage,
    msg_index: number,
    class_name: string[],
    id_prefix: string,
): Hast.HastNode[] {
    const parts = [] as Hast.HastNode[];
    const pos = node.position;
    const marker_start = msg.location.start;
    const marker_end = msg.location.end;
    const max_index = node.value.length;
    const text = node.value;

    let line = pos.start.line;
    let col = pos.start.column;
    let start_index = 0;
    let index = 0;

    function next_char() {
        const c = text[index];
        if (c === '\n') {
            ++line;
            col = 0;
        } else {
            ++col;
        }
        ++index;
    }

    function proceed(dest: Unist.Location) {
        while (line < dest.line || col < dest.column) {
            if (index >= max_index) {
                console.error('Unexpected end of text:', node, msg, 'line:', line, 'col:', col, 'index:', index);
                return false;
            }
            next_char();
        }
        next_char();
        return true;
    }

    function marker_element(value: string): Hast.Element {
        return {
            type: 'element',
            tagName: 'span',
            properties: {
                className: class_name || ['rehype-message-marker'],
                id: (id_prefix || 'rehype-message-index-') + msg_index,
                title: msg.message,
            },
            children: [{
                type: 'text',
                value: value,
            }],
            position: msg.location,
        };
    }

    // Search start position of marked area
    if (compareLocation(pos.start, marker_start) < 0) {
        // When ['text', '<span>text</span>'] or ['text', '<span>text</span>', 'text']
        // Seek the index of the end of first text
        if (!proceed(marker_start)) {
            return [];
        }

        parts.push({
            type: 'text',
            value: text.slice(0, index),
        });
        start_index = index;
    }

    if (compareLocation(marker_end, pos.end) <= 0) {
        // When ['text', '<span>text</span>'] or ['<span>text</span>']
        parts.push(marker_element(text.slice(start_index)));
        return parts;
    }

    // Search end position of marked area
    //
    // Reaches here when ['text', '<span>text</span>', 'text'] or ['<span>text</span>', 'text']

    if (!proceed(marker_end)) {
        return parts;
    }

    parts.push(marker_element(text.slice(start_index, index)));

    parts.push({
        type: 'text',
        value: text.slice(index),
    });

    return parts;
}

class Transformer {
    public messages: Unified.VMessage[];

    constructor(
        public root: Hast.Root,
        file: Unified.VFile,
        public options: MessageMarkersOptions,
    ) {
        this.messages = file.messages;
        this.messages.sort(compareMessageByStartLocation);
    }

    transform() {
        this.visit(this.root);
    }

    // Modify children for message markers.
    // When children contains marked text, this method splits the text and add proper properties to marked element.
    //
    // e.g.
    //   input:
    //     children ['This is foo text']
    //     marked text: 'foo'
    //   output:
    //     children [
    //       'This is ',
    //       <span class="..." title="...">foo<span>
    //       ' text'
    //     ]
    private visit(node: Hast.HastNode) {
        if (node.type === 'doctype' ||
            node.type === 'comment' ||
            node.type === 'text') {
            return;
        }

        if (node.children === undefined) {
            // XXX:
            // Some plugin inserts a <span> element not having 'children' property.
            return;
        }

        for (const c of node.children) {
            this.visit(c);
        }

        const push = Array.prototype.push;
        const new_children = [] as Hast.HastNode[];

        for (const child of node.children) {
            if (child.type !== 'text') {
                new_children.push(child);
                continue;
            }

            const marked = this.markText(child);
            if (marked === null) {
                new_children.push(child);
                continue;
            }

            push.apply(new_children, marked);
        }
        node.children = new_children;
    }

    private markText(node: Hast.TextNode): Hast.HastNode[] | null {
        if (node.position === undefined) {
            // XXX:
            // Some plugin inserts a <span> element not having 'position' property.
            return null;
        }

        const pos = node.position;
        for (let idx = 0; idx < this.messages.length; ++idx) {
            const msg = this.messages[idx];

            if (compareLocation(pos.end, msg.location.start) < 0) {
                // Note: this.messages were sorted by its start location
                return null;
            }

            if  (compareLocation(msg.location.end, pos.start) < 0) {
                continue;
            }

            // Note: Reaches here when a marker is partially or entirely wrapping the text

            return getMarkedParts(node, msg, idx, this.options.className, this.options.idPrefix);
        }

        return null;
    }
}

interface MessageMarkersOptions {
    className?: string[];
    idPrefix?: string;
}

export default function plugin(_: Unified.Processor, options?: MessageMarkersOptions) {
    options = options || {};

    function transformer(n: Hast.Root, f: Unified.VFile) {
        if (f.messages.length === 0) {
            return;
        }
        new Transformer(n, f, options).transform();
    }

    return transformer;
}

