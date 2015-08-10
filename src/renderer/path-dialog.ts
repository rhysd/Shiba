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

    getDialog: function() {
        return <PathDialog>document.getElementById('path-change-dialog')
    },

    onchanged: function(p: string) {
        console.log("path-dialog: Callback 'onchanged' is ignored: " + p);
    },

    open: function() {
        // Note:
        // 'chooser_opened' is a workaround to avoid calling 'onchanged'
        // callback twice.
        this.chooser_opened = false;
        this.getDialog().open();
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

    setupDialog: function(dialog) {
        dialog.addEventListener('iron-overlay-closed', () => {
            const textarea = <HTMLTextAreaElement>document.getElementById('path-text-area');
            this.path = textarea.value;
            if (!this.chooser_opened) {
                this.onchanged(this.path);
            }
        });
    },

    setupToggleButton: function() {
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

    },

    setupDropzone: function(dialog) {
        let dropzone = <HTMLDivElement>document.querySelector('.dropzone');
        const cancel_event = function(e: Event) {
            e.preventDefault();
        };
        dropzone.addEventListener('dragenter', cancel_event);
        dropzone.addEventListener('dragover', cancel_event);
        dropzone.addEventListener('drop', event => {
            event.preventDefault();
            const file: any = event.dataTransfer.files[0];
            // XXX: `path` is not standard member of `File` class
            this.path = file.path;
            this.onchanged(file.path);
            dialog.close();
        });
    },

    ready: function() {
        let dialog = this.getDialog();
        this.setupDialog(dialog);
        this.setupToggleButton();
        this.setupDropzone(dialog);
    }
});
