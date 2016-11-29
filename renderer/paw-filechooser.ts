/// <reference path="lib.d.ts" />

/* tslint:disable no-unused-variable*/
function launchFileChooser() {
    const uploader = document.querySelector('.hidden-uploader') as HTMLInputElement;
    if (uploader) {
        uploader.click();
    }
}
/* tslint:enable no-unused-variable*/

Polymer({
    is: 'paw-filechooser',

    properties: {
        onFileChosen: Object,
    },

    ready() {
        const uploader = document.querySelector('.hidden-uploader') as HTMLInputElement;
        uploader.addEventListener('change', (event: Event) => {
            const file: any = (event.target as HTMLInputElement).files[0];
            if (file !== undefined && file.path !== undefined) {
                this.onFileChosen(file.path);
            }
        });
    },
});

