import * as React from 'react';
import PathDialog from './path_dialog';
import {send} from '../ipc-send';
import log from '../log';

interface LandingProps extends React.Props<Landing> {
}

export default class Landing extends React.PureComponent<LandingProps, {}> {
    openNewTab(path: string) {
        log.debug('Will watch new path:', path);
        send('shiba:tab-opened', path);
    }

    render() {
        return (
            <div className="landing-page">
                <div>Nothing to watch. Landing page (TODO)</div>
                <PathDialog
                    fileExts={['md', 'markdown', 'mkd'] /* TODO */}
                    onOpen={this.openNewTab}
                >
                   <button>Open</button>
                </PathDialog>
            </div>
        );
    }
}
