/// <reference path="lib.d.ts" />

import {remote} from 'electron';

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
        activeIdx: {
            type: Number,
            value: 0,
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

        remote.getCurrentWebContents().on('found-in-page', (event: Event, result: FoundInPage) => {
            console.log(result, result.activeMatchOrdinal, result.matches, result.selectionArea);
            if (this.requestId !== result.requestId) {
                return;
            }
            if (result.activeMatchOrdinal) {
                this.activeIdx = result.activeMatchOrdinal;
            }
            if (result.matches) {
                this.setResult(this.activeIdx, result.matches);
            }
            if (result.finalUpdate) {
                remote.getCurrentWebContents().stopFindInPage('keepSelection');
            }
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

    search: function(word: string) {
        if (word === '') {
            this.stopSearch();
            this.focusOnInput();
            return;
        }

        if (!this.searching || this.query !== word) {
            this.requestId = remote.getCurrentWebContents().findInPage(word);
            this.searching = true;
            this.query = word;
            this.focusOnInput();
            console.log('start search: ', word, this.requestId);
            return;
        }

        // Note: When this.query === word
        this.requestId = remote.getCurrentWebContents().findInPage(word, {findNext: true});
        console.log('next search: ', word, this.requestId);
    },

    stopSearch: function() {
        if (!this.searching) {
            return;
        }
        this.searching = false;
        this.query = '';
        this.requestId = undefined;
        this.activeIdx = 0;
        remote.getCurrentWebContents().stopFindInPage('clearSelection');
        console.log('stop search: ', this.seatchWord);
    },

    setResult: function(no: number, all: number) {
        this.matches.innerText = `${no}/${all}`;
    },
});
