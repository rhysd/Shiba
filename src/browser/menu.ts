import open = require('open');
import * as Menu from 'menu';

export function build(main_window: GitHubElectron.BrowserWindow) {
    const template = [
        {
            label: 'Shiba',

            submenu: [
                {
                    label: 'Reload',
                    click: function(){ main_window.reload(); }
                },
                {
                    label: 'DevTools',
                    click: function(){ main_window.toggleDevTools(); }
                },
                {
                    label: 'Quit App',
                    accelerator: 'Command+Q',
                    click: function(){ main_window.close(); }
                },
                {
                    type: 'separator'
                },
                {
                    label: 'About Shiba',
                    click: function(){ open('https://github.com/rhysd/Shiba'); }
                }
            ]
        }
    ];

    const menu = Menu.buildFromTemplate(template);
    Menu.setApplicationMenu(menu);
    return menu;
};
