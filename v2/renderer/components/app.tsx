import * as React from 'react';
import {connect} from 'react-redux';
import {State} from '../reducers/root';
import Markdown from './markdown';
import SideMenu from './side_menu';
import Landing from './landing_page';

type AppProps = State & React.Props<App>;

export class App extends React.PureComponent<AppProps, {}> {
    render() {
        const {tabs} = this.props;
        if (tabs.currentId === null) {
            return <Landing/>;
        }

        const preview = tabs.previews.get(tabs.currentId);
        return (
            <div className="app-root">
                <SideMenu/>
                <Markdown preview={preview}/>
            </div>
        );
    }
}

function select(state: State): AppProps {
    return state;
}

export default connect(select)(App);
