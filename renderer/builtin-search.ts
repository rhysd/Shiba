/// <reference path="lib.d.ts" />

import {remote} from 'electron';
import * as path from 'path';

Polymer({
    is: 'builtin-search',

    properties: {
        displayed: {
            type: Boolean,
            value: false,
        },
        searching: {
            type: Boolean,
            value: false,
        },
        activeIdx: {
            type: Number,
            value: 0,
        },
        webviewLoaded: {
            type: Boolean,
            value: false,
        },
        onMount: Object,
        onUnmount: Object,
    },

    focusOnInput() {
        this.webview.focus();
        this.webview.send('builtin-search:focus');
    },

    _prepareWebView() {
        this.webview = document.createElement('webview') as WebViewElement;
        // Note: className is unavailable because a custom element adds style-scope to class automatically
        this.webview.style.width = '300px';
        this.webview.style.height = '80px';
        this.webview.style.outline = 'none';
        this.webview.setAttribute('nodeintegration', 'on');
        this.webview.setAttribute('autosize', 'on');
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
        this.webview.addEventListener('dom-ready', () => {
            this.webview.addEventListener('blur', (e: Event) => {
                this.focusOnInput();
            });
        });
        document.querySelector('.input-workaround').appendChild(this.webview);
    },

    ready() {
        this.button = document.querySelector('.builtin-search-button') as HTMLButtonElement;
        this.button.addEventListener('click', () => {
            this.search(this.input.value);
        });

        this.body = document.querySelector('.builtin-search-body') as HTMLDivElement;
        this.body.classList.add('animated');
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

        this.up_button = document.querySelector('.builtin-search-up') as HTMLButtonElement;
        this.up_button.addEventListener('click', () => this.searchNext(this.query, false));
        this.down_button = document.querySelector('.builtin-search-down') as HTMLButtonElement;
        this.down_button.addEventListener('click', () => this.searchNext(this.query, true));
        this.close_button = document.querySelector('.builtin-search-close') as HTMLButtonElement;
        this.close_button.addEventListener('click', () => this.dismiss());
    },

    toggle() {
        if (this.displayed) {
            this.dismiss();
        } else {
            this.show();
        }
        if (this.displayed) {
            this.focusOnInput();
        }
    },

    show() {
        if (this.displayed) {
            return;
        }

        if (!this.webviewLoaded) {
            this._prepareWebView();
            this.webviewLoaded = true;
        }

        this.body.classList.remove('slideOutUp');
        this.body.classList.add('slideInDown');
        this.body.style.display = 'block';
        this.displayed = true;
        if (this.onMount) {
            this.onMount();
        }
    },

    dismiss() {
        if (!this.displayed) {
            return;
        }

        const removeNode = () => {
            this.body.style.display = 'none';
            this.body.removeEventListener('animationend', removeNode);
            this.displayed = false;
        };
        this.body.addEventListener('animationend', removeNode);
        this.body.classList.remove('slideInDown');
        this.body.classList.add('slideOutUp');

        if (this.searching) {
            this.stopSearch();
        }
        if (this.onUnmount) {
            this.onUnmount();
        }
    },

    search(word: string) {
        if (word === '') {
            this.dismiss();
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
        this.searchNext(word, true);
    },

    searchNext(text: string, forward: boolean) {
        if (text === '') {
            return;
        }
        const options = {
            forward,
            findNext: true,
        };
        this.requestId = remote.getCurrentWebContents().findInPage(text, options);
        this.focusOnInput();
    },

    stopSearch() {
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

    setResult(no: number, all: number) {
        this.matches.innerText = `${no}/${all}`;
    },
});
