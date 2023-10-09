import * as React from 'react';
import Divider from '@mui/material/Divider';
import { WindowBar } from './WindowBar';
import { SideBar } from './SideBar';
import { Article } from './Article';
import type { MarkdownReactTree } from '../markdown';
import type { Dispatch, Heading } from '../reducer';

export interface Props {
    tree: MarkdownReactTree;
    headings: Heading[];
    path: string | null;
    dispatch: Dispatch;
}

export const Preview: React.FC<Props> = ({ tree, headings, path, dispatch }) => {
    if (tree.root === null) {
        return <></>;
    }

    return (
        <main>
            <nav aria-label="sections outline side bar">
                <WindowBar />
                <SideBar headings={headings} path={path} />
            </nav>
            <Divider orientation="vertical" />
            <Article tree={tree} dispatch={dispatch} />
        </main>
    );
};
