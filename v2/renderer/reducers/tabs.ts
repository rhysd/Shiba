import {Map} from 'immutable';
import {ActionType, ActionKind} from '../actions';
import MarkdownProcessor from '../markdown/processor';

export interface Tab {
    id: number;
    processor: MarkdownProcessor;
}

export type Tabs = Map<number, Tab>;

export interface TabsState {
    currentId: number | null;
    transformConfig: RemarkLintConfig | null;
    tabs: Tabs;
}

export const DefaultTabsState = {
    currentId: null,
    tabs: Map<number, Tab>(),
    transformConfig: null,
} as TabsState;

export default function tabs(state: TabsState = DefaultTabsState, action: ActionType): TabsState {
    switch (action.type) {
        case ActionKind.SetConfig: {
            return Object.assign({}, state, {
                transformConfig: action.config.linter.remark_lint || {},
            });
        }
        default:
            return state;
    }
}
