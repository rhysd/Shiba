import * as path from 'path';
import {app, BrowserWindow, shell} from 'electron';
import * as menu from './menu';
import loadConfig from './config';
import WatchDog from './watcher';

const loading = loadConfig().then(config => [config, new WatchDog(config)]);

// Show versions {{{
if (process.argv.indexOf('--version') !== -1) {
    const versions: any = process.versions;
    console.log(`Shiba v${app.getVersion()}: Rich markdown previewer

Usage:
  $ shiba [--detach|--version] [{direcotyr/file to watch}]

Environment:
  OS:       ${process.platform}-${process.arch}
  Electron: ${versions.electron}
  Chrome:   ${versions.chrome}
  Node.js:  ${versions.node}
`);
    app.quit();
}
// }}}

// Main Window {{{
app.on('window-all-closed', function() { app.quit(); });

app.on('ready', function() {
    loading.then((loaded: [Config, WatchDog]) => {
        const [config, dog] = loaded;
        global.config = config;

        const display_size = require('screen').getPrimaryDisplay().workAreaSize;

        function getConfigLength(key: string, default_len: number): number {
            const len = config[key];
            switch (typeof len) {
                case 'string': {
                    if (len === 'max') {
                        return display_size[key];
                    }
                    return default_len;
                }
                case 'number': {
                    return len;
                }
                default: {
                    return default_len;
                }
            }
        }

        const icon_path = path.join(__dirname, '..', '..', 'images', 'shibainu.png');
        const options: Electron.BrowserWindowOptions = {
                icon: icon_path,
                width: getConfigLength('width', 920),
                height: getConfigLength('height', 800),
            };
        if (config.hide_title_bar) {
            options.titleBarStyle = 'hidden-inset';
        }
        let win = new BrowserWindow(options);

        const html = 'file://' + path.resolve(__dirname, '..', '..', 'static', 'index.html');
        win.loadURL(html);

        dog.wakeup(win.webContents);

        win.on('closed', function(){
            win = null;
        });

        win.on('will-navigate', function(e: Event, url: string){
            e.preventDefault();
            shell.openExternal(url);
        });

        menu.build(win);

        if (process.argv[0].endsWith('Electron') && process.platform === 'darwin') {
            // Note:
            // If Shiba is run as npm package, replace dock app icon
            app.dock.setIcon(icon_path);
        }

        if (process.env.NODE_ENV === 'development') {
            win.webContents.on('devtools-opened', () => setImmediate(() => win.focus()));
            win.webContents.openDevTools({detach: true});
        }
    }).catch(e => {
        console.error('Unknown error: ', e);
        app.quit();
    });
});
// }}}

