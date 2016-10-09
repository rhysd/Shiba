interface AppConfig {
    file_ext: {
        markdown: string[];
        [n: string]: string[];
    };
    width: number | 'max';
    height: number | 'max';
    restore_window_state: boolean;
    shortcuts: {[key: string]: string};
    voice: {
        enabled: boolean;
        source: string;
    } | null;
    drawer?: {
        responsive: boolean;
    };
    linter: {
        remark_lint: {
            enabled: boolean;
            presets: string[],
            rules: string[],
        },
        proselint: {
            enabled: boolean;
            // TODO
        },
        textlint: {
            enabled: boolean;
            // TODO
        },
    };
    menu: {
        visible: boolean;
    };
    ignore_path_pattern: string;
    hide_title_bar: boolean;
    hide_menu_bar: boolean;
    preview_customize: {
        markdown?: {
            font_size: string;
            css_path: string;
            code_theme: string;
        };
    } | null;
    markdown?: {
        font_size: string;
        css_path: string;
        code_theme: string;
    };
    _config_dir_path?: string;
    [name: string]: any;
}
