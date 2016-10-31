import * as React from 'react';
import {remote} from 'electron';
import log from '../log';

interface PathDialogProps extends React.Props<PathDialog> {
    defaultPath?: string;
    fileExts: string[];
    onOpen: (path: string) => void;
    className?: string;
}

export default class PathDialog extends React.PureComponent<PathDialogProps, {}> {
    constructor(props: PathDialogProps) {
        super(props);
        this.open = this.open.bind(this);
    }

    open(e: React.MouseEvent<HTMLDivElement>) {
        e.stopPropagation();
        const path = this.props.defaultPath || process.cwd();
        log.debug('Open dialog. path:', path, ' file extensions:', this.props.fileExts);

        const filters = [
            {
                name: 'Markdown',
                extensions: this.props.fileExts,
            },
        ];

        // Note:
        // When right click, we open a dialog for a choosing file.
        // On Windows or Linux, it's not possible to open a dialog for choosing
        // files and directories.  In the case, 'openDirectory' has higher priority.
        // For the workaround, it needs to add a way to open a dialog to choose file.
        // This is a limitation of Electron framework.

        const paths = remote.dialog.showOpenDialog({
            title: 'Choose file or directory',
            defaultPath: this.props.defaultPath,
            filters,
            properties: e.button === 0 ? ['openFile', 'openDirectory'] : ['openFile'],
        });

        log.debug('Path was selected by dialog:', paths);
        if (paths && paths.length > 0) {
            this.props.onOpen(paths[0]);
        }
    }

    render() {
        return (
            <div className={this.props.className || 'path-dialog'} onClick={this.open}>
                {this.props.children}
            </div>
        );
    }
}
