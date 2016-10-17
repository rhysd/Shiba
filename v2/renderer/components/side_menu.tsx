import * as React from 'react';
import PathDialog from './path_dialog';
import {send} from '../ipc-send';
import log from '../log';

interface SideMenuProps extends React.Props<SideMenu> {
}

export default class SideMenu extends React.PureComponent<SideMenuProps, {}> {
    openNewTab(path: string) {
        log.debug('Will watch new path:', path);
        send('shiba:tab-opened', path);
    }

    render() {
        return (
            <div className="side-menu">
                Side Menu (TODO)
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
