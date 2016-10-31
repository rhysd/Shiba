import * as React from 'react';
import PathDialog from './path_dialog';
import Dropzone from './dropzone';
import {send} from '../ipc-send';
import log from '../log';

interface LandingProps extends React.Props<Landing> {
    markdownExts: string[];
}

export default class Landing extends React.PureComponent<LandingProps, {}> {
    openNewTab(path: string) {
        log.debug('Will watch new path:', path);
        send('shiba:tab-opened', path);
    }

    render() {
        return (
            <div className="landing-page">
                <Dropzone className="landing-page__dropdown" onDrop={this.openNewTab}>
                    <PathDialog
                        className="landing-page__paw-icon"
                        fileExts={this.props.markdownExts}
                        onOpen={this.openNewTab}
                    >
                        <i className="fa fa-paw fa-5x" aria-hidden="true"/>
                    </PathDialog>
                    <i className="fa fa-arrow-up fa-2x"/>
                    <div className="landing-page__message">
                        Drop <i className="fa fa-file-o" aria-hidden="true"/> / <i className="fa fa-folder-open-o" aria-hidden="true"/> or Click
                    </div>
                </Dropzone>
            </div>
        );
    }
}
