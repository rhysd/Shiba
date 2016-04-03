import {EventEmitter} from 'events';

export default class DummyWebContents extends EventEmitter {
    constructor() {
        super();
    }
    send(channel: string, ...args: any[]) {
        this.emit(channel, {/*IPC event*/}, ...args);
    }
}

