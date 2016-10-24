import {createStore} from 'redux';
import root from './reducers/root';
import * as A from './actions/type';

export type Dispatch = (action: A.Type) => void;

export default createStore(root);
