export enum ActionKind {
    SetConfig,
}

export type ActionType = {
    type: ActionKind.SetConfig;
    config: AppConfig;
};

