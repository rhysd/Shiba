import {Map} from 'immutable';
import {ActionType} from '../actions';

export interface Tab {
    id: number;
}

export type Tabs = Map<number, Tab>;

export interface TabsState {
    currentId: number | null;
    tabs: Tabs;
}

export const DefaultTabsState = {
    currentId: null,
    tabs: Map<number, Tab>(),
} as TabsState;

export default function tabs(state: TabsState = DefaultTabsState, action: ActionType): TabsState {
    console.log(action);
    return state;
}
