import * as Menu from 'menu';
import {openExternal} from 'shell';

export function build(main_window: GitHubElectron.BrowserWindow) {
    const template = [
        {
            label: 'Shiba',

            submenu: [
                {
                    label: 'Reload',
                    click: () => main_window.reload(),
                },
                {
                    label: 'DevTools',
                    click: () => main_window.toggleDevTools(),
                },
                {
                    label: 'Quit App',
                    accelerator: 'Command+Q',
                    click: () => main_window.close(),
                },
                {
                    type: 'separator'
                },
                {
                    label: 'About Shiba',
                    click: function(){ openExternal('https://github.com/rhysd/Shiba'); }
                }
            ]
        }
    ];

    const menu = Menu.buildFromTemplate(template);
    Menu.setApplicationMenu(menu);
    return menu;
};
