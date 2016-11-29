/// <reference path="lib.d.ts" />

import {shell, ipcRenderer as ipc} from 'electron';

interface LintPanelComponent extends polymer.Base {
    showLintResult(): void;
    _messagesUpdated(messages: LintMessage[]): void;
    _showLintRules(): void;
}

Polymer({
    is: 'lint-panel',

    properties: {
        messages: {
            // Note:
            // Setting many Polymer element is slow.
            // When many messages are reported from a linter, it creates many <lint-message>
            // element and calls property setter so many times.
            // When this element takes so much time, drawing preview is deferred until the messages
            // are set.
            // So we need to defer drawing messages after drawring markdown preview.  I decided to
            // call showLintResult() manually to specify the timing to draw this element.
            type: Array,
            observer: '_messagesUpdated',
        },
        lint_url: String,
        voice_src: {
            type: String,
            value: '',
        },
        enable_inset: {
            type: Boolean,
            value: false,
        },
        already_previewed: {
            type: Boolean,
            value: false,
        },
    },

    attached() {
        if (this.enable_inset && process.platform === 'darwin') {
            const header = document.getElementById('lint-header');
            header.style.textAlign = 'center';
        }
        ipc.on('shiba:return-lint-rule-url', (_: Electron.IpcRendererEvent, url: string) => {
            this.lint_url = url;
        });
        ipc.send('shiba:request-lint-rule-url');
    },

    showLintResult() {
        this.already_previewed = true;
        if (!this.messages) {
            return;
        }

        const content = document.querySelector('.lint-content');
        while (content.firstChild) {
            content.removeChild(content.firstChild);
        }

        for (const m of this.messages) {
            const msg = document.createElement('lint-message') as LintMessageElement;
            msg.header = m.header;
            msg.body = m.body;
            content.appendChild(msg);
        }

        const header = document.getElementById('lint-header');
        if (this.messages.length > 0) {
            header.innerText = 'Error';
            header.setAttribute('error', '');
            if (this.voice_src !== '') {
                const voice = document.querySelector('.voice-notification') as HTMLAudioElement;
                if (voice) {
                    voice.play();
                }
            }
        } else {
            header.innerText = 'No Error';
            header.setAttribute('no_error', '');
        }
    },

    _messagesUpdated(messages: LintMessage[]) {
        if (this.messages && this.already_previewed) {
            this.showLintResult();
        }
    },

    _showLintRules() {
        if (this.lint_url === undefined) {
            console.log('No lint URL');
            return;
        }

        shell.openExternal(this.lint_url);
    },
} as LintPanelComponent);
