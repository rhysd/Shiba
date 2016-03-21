interface FoundInPage {
    requestId: number;
    finalUpdate: boolean;
    activeMatchOrdinal?: number;
    matches?: number;
    selectionArea: Object;
}

export default class BuiltinSearch {
    private requestId: number;
    private activeMatchIdx: number;

    constructor(private listener: Electron.WebContents) {
        this.listener.on('found-in-page', this.matchFound.bind(this));
        this.listener.on('ipc-message', this.onIpcMessage.bind(this));
    }

    startFinding(text: string) {
        console.log('start!: ', text);
        this.requestId = this.listener.findInPage(text);
    }

    stopFinding() {
        console.log('stop!: ', this.requestId);
        this.listener.stopFindInPage('clearSelection');
    }

    findNext(text: string, forward: boolean) {
        console.log('next!:', text);
        this.requestId = this.listener.findInPage(text, {
            forward,
            findNext: true,
        });
    }

    sendMatchResult(active: number, all: number) {
        this.listener.send('builtin-search:match-result', active, all);
    }

    onIpcMessage(event: Event, args: any[]) {
        console.log('builtin-search:onIpcMessage: ', args);
        const channel = args[0] as string;
        switch (channel) {
            case 'builtin-search:start-finding': {
                this.startFinding(args[1] as string);
                break;
            }

            case 'builtin-search:stop-finding': {
                this.stopFinding();
                break;
            }

            case 'builtin-search:find-next': {
                this.findNext(args[1] as string, args[2] as boolean);
                break;
            }

            default:
                break;
        }
    }

    matchFound(event: Event, result: FoundInPage) {
        console.log(result, result.activeMatchOrdinal, result.matches, result.selectionArea);
        if (this.requestId !== result.requestId) {
            return;
        }
        if (result.activeMatchOrdinal) {
            this.activeMatchIdx = result.activeMatchOrdinal;
        }
        if (result.finalUpdate && result.matches !== undefined) {
            this.sendMatchResult(this.activeMatchIdx, result.matches);
        }
    }
}
