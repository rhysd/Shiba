import * as React from 'react';
import {Tab} from '../reducers/tabs';

interface MarkdownProps extends React.Props<Markdown> {
    tab: Tab;
}

export default class Markdown extends React.PureComponent<MarkdownProps, {}> {
    render() {
        if (this.props.tab.preview === null) {
            return <div>Watching directory.</div>;
        }
        return <div className="markdown-body">
            {this.props.tab.preview}
        </div>;
    }
}
