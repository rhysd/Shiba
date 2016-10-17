import {Map} from 'immutable';
import {ReactElement} from 'react';
import {ActionType, ActionKind} from '../actions';
import MarkdownProcessor from '../markdown/processor';

export interface Preview {
    id: number | null;
    processor: MarkdownProcessor;
    watchingPath: string;
    contents: ReactElement<any> | null;
}

export type Previews = Map<number, Preview>;

export interface TabsState {
    currentId: number | null;
    transformConfig: RemarkLintConfig | null;
    mdExtensions: string[];
    previews: Previews;
}

export const DefaultTabsState = {
    currentId: null,
    previews: Map<number, Preview>(),
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
                currentId: action.preview.id, // Focus to new tab
                previews: state.previews.set(action.preview.id, action.preview),
            });
        }
        case ActionKind.UpdatePreview: {
            return Object.assign({}, state, {
                previews: state.previews.set(
                    action.id,
                    Object.assign({}, state.previews.get(action.id), {
                        contents: action.contents,
                    }),
                ),
            });
        }
        default:
            return state;
    }
}
