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
                    click: () => main_window.webContents.openDevTools({detach: true}),
                },
                {
                    label: 'Quit App',
                    accelerator: 'Command+Q',
                    click: () => main_window.close(),
                },
                {
                    type: 'separator',
                },
                {
                    label: 'About Shiba',
                    click: function(){ shell.openExternal('https://github.com/rhysd/Shiba'); },
                },
            ],
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
                    type: 'separator',
                },
                {
                    label: 'Choose File',
                    click: () => main_window.webContents.send('shiba:choose-file'),
                },
                {
                    label: 'Lint Result',
                    click: () => main_window.webContents.send('shiba:lint'),
                },
                {
                    label: 'Outline',
                    click: () => main_window.webContents.send('shiba:outline'),
                },
                {
                    label: 'Search',
                    click: () => main_window.webContents.send('shiba:search'),
                },
                {
                    label: 'Reload',
                    click: () => main_window.webContents.send('shiba:reload'),
                },
                {
                    label: 'Print',
                    click: () => main_window.webContents.print(),
                },
            ],
        },
    ];

    const menu = Menu.buildFromTemplate(template);
    Menu.setApplicationMenu(menu);
    return menu;
};
