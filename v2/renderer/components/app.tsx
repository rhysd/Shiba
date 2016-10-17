import * as React from 'react';
import {connect} from 'react-redux';
import {State} from '../reducers/root';
import Markdown from './markdown';

type AppProps = State & React.Props<App>;

export class App extends React.PureComponent<AppProps, {}> {
    render() {
        const {tabs} = this.props;
        if (tabs.currentId === null) {
            return <div>Nothing to watch. Landing page (TODO)</div>;
        }

        const preview = tabs.previews.get(tabs.currentId);
        return (
            <Markdown preview={preview}/>
        );
    }
}

function select(state: State): AppProps {
    return state;
}

export default connect(select)(App);
