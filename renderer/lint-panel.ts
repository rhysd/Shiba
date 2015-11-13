/// <reference path="lib.d.ts" />

interface LintPanelComponent extends polymer.Base {
    _setMessages(messages: LintMessage[]) : void;
    _contentUpdated(messages: LintMessage[]) : void;
    _showLintRules() : void;
}

Polymer({
    is: 'lint-panel',

    properties: {
        content: {
            type: Array,
            observer: '_contentUpdated'
        },
        lint_url: String,
        voice_src: {
            type: String,
            value: ''
        }
    },

    _setMessages(messages: LintMessage[]) {
        const content = document.querySelector('.lint-content');
        while (content.firstChild) {
            content.removeChild(content.firstChild);
        }

        for (const m of messages) {
            let msg = document.createElement('lint-message') as LintMessageElement;
            msg.header = m.header;
            msg.body = m.body;
            content.appendChild(msg);
        }
    },

    _contentUpdated: function(messages: LintMessage[]) {
        this._setMessages(messages);

        const header = document.getElementById('lint-header');
        if (messages.length > 0) {
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

    _showLintRules: function() {
        if (this.lint_url === undefined) {
            console.log('No lint URL');
            return;
        }

        this.openExternal = this.openExternal || require('shell').openExternal as (url: string) => void;
        this.openExternal(this.lint_url);
    }
} as LintPanelComponent);
