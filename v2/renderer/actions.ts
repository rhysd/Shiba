import {ReactElement} from 'react';
import {Preview} from './reducers/tabs';

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
    preview: Preview;
} | {
    type: ActionKind.UpdatePreview;
    id: number;
    contents: ReactElement<any> | null;  // When the updated tab is in background, rendering preview will be skipped as null.
};

