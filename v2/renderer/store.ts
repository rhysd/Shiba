import {createStore} from 'redux';
import root from './reducers/root';
import {ActionType} from './actions';

export type Dispatch = (action: ActionType) => void;

export default createStore(root);
