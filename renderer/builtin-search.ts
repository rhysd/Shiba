/// <reference path="lib.d.ts" />

import {ipcRenderer as ipc} from 'electron';

Polymer({
    is: 'builtin-search',

    properties: {
        displayed: {
            type: Boolean,
            value: true,
        },
        searching: {
            type: Boolean,
            value: false,
        },
    },

    ready: function() {
        this.input = document.querySelector('.builtin-search-input') as HTMLInputElement;
        this.input.addEventListener('keypress', (event: KeyboardEvent & {keyIdentifier: string}) => {
            if (event.keyIdentifier === 'Enter') {
                this.search(this.input.value);
            }
        });
        this.input.addEventListener('blur', (e: Event) => {
            e.preventDefault();
            e.stopPropagation();
            this.focusOnInput();
        });

        this.button = document.querySelector('.builtin-search-button') as HTMLButtonElement;
        this.button.addEventListener('click', () => {
            this.search(this.input.value);
        });

        this.body = document.querySelector('.builtin-search-body') as HTMLDivElement;
        if (this.displayed) {
            this.body.style.display = 'block';
        }

        this.matches = document.querySelector('.builtin-search-matches') as HTMLDivElement;

        ipc.on('builtin-search:match-result', (event: Event, active: number, all: number) => {
            this.setResult(active, all);
        });
    },

    focusOnInput() {
        this.input.$.input.focus();
    },

    show: function() {
        if (this.displayed) {
            return;
        }

        this.body.style = 'block';
        this.displayed = true;
    },

    dismiss: function() {
        if (!this.displayed) {
            return;
        }

        this.body.style = 'none';
        this.displayed = false;

        if (this.searching) {
            this.stopSearch();
        }
    },

    search: function(text: string) {
        if (text === '') {
            this.stopSearch();
            this.focusOnInput();
            return;
        }

        if (!this.searching || this.query !== text) {
            ipc.send('builtin-search:start-finding', text);
            this.searching = true;
            this.query = text;
            this.focusOnInput();
            console.log('start search: ', text);
            return;
        }

        // Note: When this.query === text
        ipc.send('builtin-search:find-next', text, /*forward: */true);
        console.log('next search: ', text);
    },

    stopSearch: function() {
        if (!this.searching) {
            return;
        }
        this.searching = false;
        this.query = '';
        ipc.send('builtin-search:stop-finding');
        console.log('stop search: ', this.seatchWord);
    },

    setResult: function(no: number, all: number) {
        this.matches.innerText = `${no}/${all}`;
    },
});
