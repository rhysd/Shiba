import {ReactElement} from 'react';
import {Preview} from '../reducers/tabs';

export enum Kind {
    SetConfig,
    NewTab,
    UpdatePreview,
}

export type Type = {
    type: Kind.SetConfig;
    config: AppConfig;
} | {
    type: Kind.NewTab;
    preview: Preview;
} | {
    type: Kind.UpdatePreview;
    id: number;
    contents: ReactElement<any> | null;  // When the updated tab is in background, rendering preview will be skipped as null.
};

