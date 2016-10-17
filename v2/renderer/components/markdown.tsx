import * as React from 'react';
import {Preview} from '../reducers/tabs';

interface MarkdownProps extends React.Props<Markdown> {
    preview: Preview;
}

export default class Markdown extends React.PureComponent<MarkdownProps, {}> {
    render() {
        if (this.props.preview.contents === null) {
            return <div>Watching directory.</div>;
        }
        return <div className="markdown-body">
            {this.props.preview.contents}
        </div>;
    }
}
