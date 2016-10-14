import * as React from 'react';
import PathDialog from './path_dialog';
import MarkdownProcessor from '../markdown/processor';
import log from '../log';

const Processor = new MarkdownProcessor();

interface Props extends React.Props<MarkdownTest> {
}

interface State {
    preview: React.ReactElement<any> | null;
}

export default class MarkdownTest extends React.Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            preview: null,
        };
        this.showPreview = this.showPreview.bind(this);
    }

    showPreview(file: string) {
        Processor.processFile(file).then(vfile => {
            this.setState({preview: vfile.contents});
        }).catch(err => {
            log.error(err);
        });
    }

    componentDidMount() {
        this.showPreview('./foo.md');
    }

    render() {
        return <div>
            <PathDialog fileExts={['md', 'markdown', 'mkd']} onOpen={this.showPreview}>
                <button>Choose Path</button>
            </PathDialog>
            <div className="markdown-body">
                {this.state.preview}
            </div>
        </div>;
    }
}
