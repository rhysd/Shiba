import {combineReducers} from 'redux';
import tabs, {TabsState} from './tabs';

export type State = {
    tabs: TabsState,
};

const root = combineReducers<State>({
    tabs,
});
export default root;
