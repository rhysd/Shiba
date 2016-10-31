import * as fs from 'fs';
import * as React from 'react';
import log from '../log';

interface DropzoneProps extends React.Props<Dropzone> {
    onDrop: (path: string) => void;
    className?: string;
}

export default class Dropzone extends React.PureComponent<DropzoneProps, {}> {
    zone: HTMLDivElement;

    handleDrop(e: DragEvent) {
        e.preventDefault();
        e.stopPropagation();

        const files = e.dataTransfer.files;
        if (!files || files.length === 0) {
            log.error('File location could not get from event:', e);
            return;
        }

        const file = files[0].path;
        if (!file) {
            log.error('File path is empty:', e);
        }

        fs.stat(file, (err, _) => {
            if (err) {
                log.error('Dropped file/directory not found', err);
                return;
            }

            this.props.onDrop(file);
        });
    }

    cancelEvent(e: Event) {
        e.preventDefault();
        e.stopPropagation();
    }

    componentDidMount() {
        this.zone.addEventListener('dragenter', this.cancelEvent);
        this.zone.addEventListener('dragover', this.cancelEvent);
        this.zone.addEventListener('drop', this.handleDrop.bind(this));
    }

    render() {
        return (
            <div
                className={this.props.className || 'dropzone'}
                ref={r => { this.zone = r; }}
            >
                {this.props.children}
            </div>
        );
    }
}
