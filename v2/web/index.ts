import { marked } from 'marked';
import hljs from 'highlight.js';

interface Ipc {
    postMessage(m: string): void;
}

declare global {
    interface Window {
        myMarkdownPreview: MyPreviewApp;
        ipc: Ipc;
    }
}

type MessageFromMain = {
    kind: 'content';
    content: string;
};

type MessageToMain =
    | {
          kind: 'init';
      }
    | {
          kind: 'open';
          link: string;
      };

function sendMessage(m: MessageToMain): void {
    window.ipc.postMessage(JSON.stringify(m));
}

const RE_ANCHOR_START = /^<a /;

class MyRenderer extends marked.Renderer {
    override link(href: string, title: string, text: string): string {
        const rendered = super.link(href, title, text);
        return rendered.replace(RE_ANCHOR_START, '<a onclick="window.myMarkdownPreview.onLinkClicked(event)" ');
    }
}

marked.setOptions({
    renderer: new MyRenderer(),
    highlight: (code, lang) => {
        const language = hljs.getLanguage(lang) ? lang : 'plaintext';
        return hljs.highlight(code, { language }).value;
    },
    langPrefix: 'hljs language-',
    gfm: true,
});

class MyPreviewApp {
    receive(msg: MessageFromMain): void {
        switch (msg.kind) {
            case 'content':
                const elem = document.getElementById('preview');
                if (elem === null) {
                    console.error("'preview' element is not found");
                    return;
                }
                elem.innerHTML = marked.parse(msg.content);
                break;
            default:
                console.error('Unknown message:', msg);
                break;
        }
    }

    onLinkClicked(event: MouseEvent): void {
        event.preventDefault();
        if (event.target === null) {
            return;
        }
        const a = event.target as HTMLAnchorElement;
        const link = a.getAttribute('href');
        if (!link) {
            return;
        }
        sendMessage({
            kind: 'open',
            link,
        });
    }
}

window.myMarkdownPreview = new MyPreviewApp();
sendMessage({ kind: 'init' });
