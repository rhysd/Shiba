import {Map} from 'immutable';
import {ReactElement} from 'react';
import {ActionType, ActionKind} from '../actions';
import MarkdownProcessor from '../markdown/processor';

export interface Tab {
    id: number | null;
    processor: MarkdownProcessor;
    watchingPath: string;
    preview: ReactElement<any> | null;
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
        case ActionKind.NewTab: {
            return Object.assign({}, state, {
                currentId: action.id, // Focus to new tab
                tabs: state.tabs.set(action.id, {
                    id: action.id,
                    processor: new MarkdownProcessor(
                        Object.assign({}, (state.transformConfig || {}), action.config)
                    ),
                    watchingPath: action.path,
                    preview: null,
                }),
            });
        }
        default:
            return state;
    }
}
