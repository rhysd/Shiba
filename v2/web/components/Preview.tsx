import * as React from 'react';
import Divider from '@mui/material/Divider';
import { Resizable } from 're-resizable';
import { WindowBar } from './WindowBar';
import { SideBar } from './SideBar';
import { Article } from './Article';
import type { MarkdownReactTree } from '../markdown';
import type { Dispatch, Heading } from '../reducer';

const NAV_RESIZE_DIRECTION = {
    top: false,
    right: true,
    bottom: false,
    left: false,
    topRight: false,
    bottomRight: false,
    bottomLeft: false,
    topLeft: false,
};

const NAV_DEFAULT_SIZE = {
    width: '20%',
    height: '100%',
};

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
            <Resizable defaultSize={NAV_DEFAULT_SIZE} minWidth="200px" enable={NAV_RESIZE_DIRECTION} as="nav">
                <WindowBar />
                <SideBar headings={headings} path={path} />
            </Resizable>
            <Divider orientation="vertical" />
            <Article tree={tree} dispatch={dispatch} />
        </main>
    );
};
