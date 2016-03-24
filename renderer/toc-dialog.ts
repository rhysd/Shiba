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
            // TODO: Prepare contents
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
