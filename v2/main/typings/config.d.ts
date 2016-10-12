interface RemarkLintConfig {
    enabled?: boolean;
    presets?: string[];
    rules?: string[];
}

interface RedpenConfig {
    enabled?: boolean;
    server_command?: string;
    port?: number;
    // TODO
}

interface TextLintConfig {
    enabled?: boolean;
    // TODO
}

interface ProseLintConfig {
    enabled?: boolean;
    command?: string;
    // TODO
}

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
        remark_lint?: RemarkLintConfig,
        redpen?: RedpenConfig,
        textlint?: TextLintConfig,
        proselint?: ProseLintConfig,
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
