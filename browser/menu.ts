import {shell, Menu} from 'electron';

export function build(main_window: Electron.BrowserWindow) {
    const template = [
        {
            label: 'Shiba',

            submenu: [
                {
                    label: 'Restart',
                    click: () => main_window.reload(),
                },
                {
                    label: 'DevTools',
                    click: () => main_window.webContents.toggleDevTools(),
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
                    click: function(){ shell.openExternal('https://github.com/rhysd/Shiba'); }
                }
            ]
        },

        {
            label: 'Actions',
            submenu: [
                {
                    label: 'Copy',
                    accelerator: 'Command+C',
                    role: 'copy',
                },
                {
                    label: 'Select All',
                    accelerator: 'Command+A',
                    role: 'selectall',
                },
                {
                    type: 'separator'
                },
            ],
        },
    ];

    const menu = Menu.buildFromTemplate(template);
    Menu.setApplicationMenu(menu);
    return menu;
};
