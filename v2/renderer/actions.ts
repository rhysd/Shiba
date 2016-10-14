import {Tab} from './reducers/tabs';

export enum ActionKind {
    SetConfig,
    NewTab,
}

export type ActionType = {
    type: ActionKind.SetConfig;
    config: AppConfig;
} | {
    type: ActionKind.NewTab;
    tab: Tab;
};

