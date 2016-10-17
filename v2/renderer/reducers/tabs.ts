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
    mdExtensions: string[];
    tabs: Tabs;
}

export const DefaultTabsState = {
    currentId: null,
    tabs: Map<number, Tab>(),
    transformConfig: null,
    mdExtensions: [],
} as TabsState;

export default function tabs(state: TabsState = DefaultTabsState, action: ActionType): TabsState {
    switch (action.type) {
        case ActionKind.SetConfig: {
            return Object.assign({}, state, {
                transformConfig: action.config.linter.remark_lint || {},
                mdExtensions: action.config.file_ext.markdown || ['md', 'markdown', 'mkd'],
            });
        }
        case ActionKind.NewTab: {
            return Object.assign({}, state, {
                currentId: action.tab.id, // Focus to new tab
                tabs: state.tabs.set(action.tab.id, action.tab),
            });
        }
        case ActionKind.UpdatePreview: {
            return Object.assign({}, state, {
                tabs: state.tabs.set(
                    action.id,
                    Object.assign({}, state.tabs.get(action.id), {
                        preview: action.preview,
                    }),
                ),
            });
        }
        default:
            return state;
    }
}
