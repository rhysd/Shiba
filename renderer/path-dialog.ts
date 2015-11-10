/// <reference path="lib.d.ts" />

interface PathDialogComponent extends polymer.Base {
    getDialog(): PathDialog;
    onchanged(p: string): void;
    open(): void;
    onChooserLaunched(): void;
    onFileSpecified(): void;
    setupDialog(dialog: PathDialog): void;
    setupToggleButton(): void;
}

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
        const hidden_input = document.getElementById('path-hidden-input') as HTMLInputElement;
        // Note: Electron extends 'path' property in File API
        const path = (hidden_input.files[0] as any).path;
        const text_area = document.getElementById('path-text-area') as HTMLTextAreaElement;
        text_area.value = path;
        this.path = path;
        this.onchanged(path);
    },

    setupDialog: function(dialog) {
        dialog.addEventListener('iron-overlay-closed', () => {
            const textarea = document.getElementById('path-text-area') as HTMLTextAreaElement;
            this.path = textarea.value;
            if (!this.chooser_opened) {
                this.onchanged(this.path);
            }
        });
    },

    setupToggleButton: function() {
        const toggle = document.getElementById('want-to-choose-dir-button');
        toggle.addEventListener('change', function() {
            const hidden_input = document.getElementById('path-hidden-input');
            if (this.checked) {
                hidden_input.setAttribute('webkitdirectory', '');
                hidden_input.setAttribute('directory', '');
            } else {
                hidden_input.removeAttribute('webkitdirectory');
                hidden_input.removeAttribute('directory');
            }
        });

    },

    ready: function() {
        const dialog = this.getDialog();
        this.setupDialog(dialog);
        this.setupToggleButton();
    }
} as PathDialogComponent);
