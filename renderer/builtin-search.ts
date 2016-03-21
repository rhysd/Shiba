/// <reference path="lib.d.ts" />

import {remote} from 'electron';
import * as path from 'path';

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

    focusOnInput() {
        this.webview.focus();
        this.webview.send('builtin-search:focus');
    },

    ready: function() {
        this.webview = document.querySelector('.input-workaround') as HTMLElement;
        this.webview.src = 'file://' + path.join(__dirname, 'search-input.html');
        this.webview.addEventListener('ipc-message', (e: any) => {
            const channel = e.channel as string;
            switch (channel) {
                case 'builtin-search:query': {
                    const text = (e.args[0] || '') as string;
                    this.search(text);
                    break;
                }
                case 'builtin-search:close': {
                    this.dismiss();
                    break;
                }
                default:
                    break;
            }
        });
        this.webview.addEventListener('console-message', (e: any) => {
            console.log('console-message: ', e.line + ': ' + e.message);
        });

        this.webview.addEventListener('dom-ready', () => {
            this.webview.addEventListener('blur', (e: Event) => {
                this.focusOnInput();
            });
            if (this.displayed) {
                this.focusOnInput();
            }
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
            if (this.requestId !== result.requestId) {
                return;
            }
            if (result.activeMatchOrdinal) {
                this.activeIdx = result.activeMatchOrdinal;
            }
            if (result.finalUpdate && result.matches) {
                this.setResult(this.activeIdx, result.matches);
            }
        });
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

        console.log('dismiss!');

        this.body.style.display = 'none';
        this.displayed = false;

        if (this.searching) {
            this.stopSearch();
        }
    },

    search: function(word: string) {
        if (word === '') {
            this.stopSearch();
            return;
        }

        if (!this.searching || this.query !== word) {
            this.requestId = remote.getCurrentWebContents().findInPage(word);
            this.searching = true;
            this.query = word;
            this.focusOnInput();
            return;
        }

        // Note: When this.query === word
        this.requestId = remote.getCurrentWebContents().findInPage(word, {findNext: true});
        this.focusOnInput();
    },

    stopSearch: function() {
        if (!this.searching) {
            return;
        }
        this.setResult(0, 0);
        remote.getCurrentWebContents().stopFindInPage('clearSelection');
        this.searching = false;
        this.query = '';
        this.requestId = undefined;
        this.activeIdx = 0;
    },

    setResult: function(no: number, all: number) {
        this.matches.innerText = `${no}/${all}`;
    },
});
