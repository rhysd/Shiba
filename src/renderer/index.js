'use strict';

let remote = require('remote');

function onPathButtonPushed() {
    document.getElementById('path-change').open();
}

function makeTitle(path) {
    if (path === '') {
        return 'Shiba';
    } else {
        return 'Shiba (' + path + ')';
    }
}

function getScroller() {
    const selected = document.getElementById('main-drawer').selected;
    if (selected === null) {
        return null;
    }

    return document.querySelector('paper-header-panel[' + selected + ']').scroller;
}

function scrollContentBy(x, y) {
    let scroller = getScroller();
    if (!scroller) {
        return;
    }

    if (x !== 0) {
        scroller.scrollLeft += x;
    }
    if (y !== 0) {
        scroller.scrollTop += y;
    }
}

window.onload = function(){
    const path = remote.require('./initial_path.js')();

    let Watcher = remote.require('./watcher.js');
    var watcher = new Watcher(
        path,

        // Markdown renderer
        function(html) {
            document.querySelector('markdown-area').content = html;
        },

        // Linter result renderer
        function(messages) {
            document.getElementById('lint-area').content = messages;
            let button = document.getElementById('lint-button');
            if (messages.length === 0) {
                button.style.color = '#d99e5f';
            } else {
                button.style.color = '#ce3c4a';
            }
        }
    );

    document.getElementById('lint-area').lint_url = watcher.getLintRuleURL();

    let dialog = document.getElementById('path-change');
    dialog.path = path;
    dialog.onchanged = function(path) {
        watcher.changeWatchingDir(path);
        document.title = makeTitle(path);
    };

    if (path === '') {
        dialog.open();
    }
    document.title = makeTitle(path);

    let key_receiver = new KeyReceiver();

    key_receiver.on('Lint', function() {
            document.getElementById('main-drawer').togglePanel();
    });

    let scroller = document.querySelector('paper-header-panel[main]').scroller;

    key_receiver.on('PageUp', function() {
            scrollContentBy(0, -window.innerHeight / 2);
    });

    key_receiver.on('PageDown', function() {
            scrollContentBy(0, window.innerHeight / 2);
    });

    key_receiver.on('PageLeft', function() {
            scrollContentBy(-window.innerHeight / 2, 0);
    });

    key_receiver.on('PageRight', function() {
            scrollContentBy(window.innerHeight / 2, 0);
    });

    key_receiver.on('PageTop', function() {
            let scroller = getScroller();
            if (scroller) {
                scroller.scrollTop = 0;
            }
    });

    key_receiver.on('PageBottom', function() {
            let scroller = getScroller();
            if (scroller) {
                scroller.scrollTop = scroller.scrollHeight;
            }
    });

    key_receiver.on('ChangePath', function() {
            dialog.open();
    });
}
