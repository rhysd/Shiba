import React from 'react';
import { Search } from './Search';
import type { Dispatch, State } from '../reducer';

interface Props {
    dispatch: Dispatch;
    state: State;
}

export const Preview: React.FC<Props> = ({state, dispatch}) => {
    const { preview } = state;
    return <>
        {state.search && preview ? <Search previewContent={preview.hast} dispatch={dispatch}/> : undefined}
        <article className="markdown-body">{preview?.react}</article>
    </>;
};
