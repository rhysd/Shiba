/// <reference path="lib.d.ts" />

Polymer({
    is: 'toc-dialog',

    properties: {
        opened: {
            type: Boolean,
            value: false,
        },
        innerDialog: Object,
    },

    open: function(outline: Heading[]) {
        if (this.innerDialog) {
            if (outline.length > 0) {
                const elems = outline.map(h => {
                    const outer = document.createElement('paper-item');
                    const inner = document.createElement('paper-item-body');
                    const header = document.createElement('h' + h.level);
                    header.innerText = `${'#'.repeat(h.level)} ${h.title}`;
                    // outer.addEventListener('click', () => {
                    //     // TODO
                    //     console.log('clicked: ', h.hash);
                    // });
                    inner.appendChild(header);
                    outer.appendChild(inner);
                    return outer;
                });
                (elems[0] as any).focused = true;

                const listbox = document.getElementById('toc-listbox');
                for (const e of elems) {
                    listbox.appendChild(e);
                }
            }
            this.innerDialog.open();
            this.opened = true;
        }
    },

    close: function() {
        if (this.innerDialog) {
            this.innerDialog.close();
            this.opened = false;
        }
    },

    toggle: function(outline: Heading[] = []) {
        if (this.opened) {
            this.close();
        } else {
            this.open(outline);
        }
    },

    ready: function() {
        this.innerDialog = (document.getElementById('toc-body') as any) as PaperDialogElement;
    },
});
