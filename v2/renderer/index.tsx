import * as React from 'react';
import * as ReactDOM from 'react-dom';
import MarkdownTest from './components/markdown_test';
import * as ipc from './external_input/ipc';

ipc.setupReceivers();
ipc.send('shiba:request-config');

ReactDOM.render(
    <MarkdownTest/>,
    document.getElementById('shiba-app')
);
