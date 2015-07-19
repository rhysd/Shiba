import Menu = require('menu');
import open = require('open');

export function build(main_window) {
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
