/// <reference path="lib.d.ts" />
/// <reference path="../../typings/polymer/polymer.d.ts" />

Polymer({
    is: 'path-dialog',

    properties: {
        open: Object,

        onchanged: Object,

        label: {
            type: String,
            value: ""
        },

        path: {
            type: String,
            value: ""
        }
    },

    onchanged: function(p: string) {
        console.log("path-dialog: Callback 'onchanged' is ignored: " + p);
    },

    open: function() {
        // Note:
        // 'chooser_opened' is a workaround to avoid calling 'onchanged'
        // callback twice.
        this.chooser_opened = false;
        (<PathDialog>document.getElementById('path-change-dialog')).open();
    },

    onChooserLaunched: function() {
        this.chooser_opened = true;
        document.getElementById('path-hidden-input').click();
    },

    onFileSpecified: function() {
        // XXX: 'path' doesn't exist in standard File object?
        const path = (<any>document.getElementById('path-hidden-input')).files[0].path;
        (<HTMLTextAreaElement>document.getElementById('path-text-area')).value = path;
        this.path = path;
        this.onchanged(path);
    },

    ready: function() {
        let that = this;

        let dialog = <PathDialog>document.getElementById('path-change');
        dialog.addEventListener('iron-overlay-closed', function() {
            const textarea = <HTMLTextAreaElement>document.getElementById('path-text-area');
            that.path = textarea.value;
            if (!that.chooser_opened) {
                that.onchanged(that.path);
            }
        });

        let toggle = document.getElementById('want-to-choose-dir-button');
        toggle.addEventListener('change', function() {
            let hidden_input = document.getElementById('path-hidden-input');
            if (this.checked) {
                hidden_input.setAttribute('webkitdirectory', '');
                hidden_input.setAttribute('directory', '');
            } else {
                hidden_input.removeAttribute('webkitdirectory');
                hidden_input.removeAttribute('directory');
            }
        });
    }
});
