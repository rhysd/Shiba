export enum ActionKind {
    SetConfig,
    NewTab,
}

export type ActionType = {
    type: ActionKind.SetConfig;
    config: AppConfig;
} | {
    type: ActionKind.NewTab;
    config: RemarkLintConfig;
    id: number;
    path: string;
};

