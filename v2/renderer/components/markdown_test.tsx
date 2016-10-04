import * as React from 'react';
import MarkdownProcessor from '../markdown/processor';

const Processor = new MarkdownProcessor();

interface Props extends React.Props<MarkdownTest> {
}

interface State {
    preview: React.ReactElement<any> | null;
}

export default class MarkdownTest extends React.Component<Props, State> {
    refs: {
        input: HTMLInputElement;
    };

    constructor(props: Props) {
        super(props);
        this.state = {
            preview: null,
        };
    }

    showPreview(file: string) {
        Processor.processFile(file).then(elems => {
            this.setState({preview: elems});
        }).catch(err => {
            console.error(err);
        });
    }

    componentDidMount() {
        this.refs.input.addEventListener('keydown', (e: KeyboardEvent) => {
            if (e.key === 'Enter') {
                const i = e.target as HTMLInputElement;
                this.showPreview(i.value);
            }
        });
        this.showPreview('./foo.md');
    }

    render() {
        return <div>
            <input ref="input" />
            <div className="markdown-body">
                {this.state.preview}
            </div>
        </div>;
    }
}
