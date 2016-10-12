import * as React from 'react';
import * as ReactDOM from 'react-dom';
import {Provider} from 'react-redux';
import MarkdownTest from './components/markdown_test';
import * as ipc from './external_input/ipc';
import {send} from './ipc-send';
import Store from './store';

ipc.setupReceivers();
send('shiba:request-config');

ReactDOM.render(
    <Provider store={Store}>
        <MarkdownTest/>,
    </Provider>,
    document.getElementById('shiba-app')
);
