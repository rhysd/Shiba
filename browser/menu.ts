import * as path from 'path';
import {Menu} from 'electron';
import openAboutWindow from 'about-window';

export function build(main_window: Electron.BrowserWindow) {
    const template = [
        {
            label: 'Shiba',

            submenu: [
                {
                    label: 'About Shiba',
                    click: () => openAboutWindow({
                        icon_path: path.join(__dirname, '..', '..', 'images', 'shibainu.png'),
                        copyright: 'Copyright (c) 2015 rhysd',
                    }),
                },
                {
                    type: 'separator',
                },
                {
                    label: 'Restart',
                    click: () => main_window.reload(),
                },
                {
                    label: 'DevTools',
                    click: () => main_window.webContents.openDevTools({mode: 'detach'}),
                },
                {
                    label: 'Quit',
                    accelerator: 'Command+Q',
                    click: () => main_window.close(),
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
    ] as Electron.MenuItemConstructorOptions[];

    const menu = Menu.buildFromTemplate(template);
    Menu.setApplicationMenu(menu);
    return menu;
}
