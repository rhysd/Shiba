/// <reference path="lib.d.ts" />
/// <reference path="../../typings/polymer/polymer.d.ts" />

function launchFileChooser() {
    const uploader = <HTMLInputElement>document.querySelector('.hidden-uploader');
    if (uploader) {
        uploader.click();
    }
}

Polymer({
    is: 'paw-filechooser',

    properties: {
        onFileChosen: Object,
    },

    ready: function() {
        let uploader = <HTMLInputElement>document.querySelector('.hidden-uploader');
        uploader.addEventListener('change', (event: Event) => {
            const file: any = (<HTMLInputElement>event.target).files[0];
            if (file !== undefined && file.path !== undefined) {
                this.onFileChosen(file.path);
            }
        });
    },
});

