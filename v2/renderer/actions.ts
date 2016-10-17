import {ReactElement} from 'react';
import {Tab} from './reducers/tabs';

export enum ActionKind {
    SetConfig,
    NewTab,
    UpdatePreview,
}

export type ActionType = {
    type: ActionKind.SetConfig;
    config: AppConfig;
} | {
    type: ActionKind.NewTab;
    tab: Tab;
} | {
    type: ActionKind.UpdatePreview;
    id: number;
    preview: ReactElement<any> | null;  // When the updated tab is in background, rendering preview will be skipped as null.
};

