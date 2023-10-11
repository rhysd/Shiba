import * as React from 'react';
import Box from '@mui/material/Box';
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

const SX_NON_VIBRANT = {
    bgcolor: 'background.paper',
};

export interface Props {
    tree: MarkdownReactTree;
    headings: Heading[];
    path: string | null;
    titleBar: boolean;
    vibrant: boolean;
    dispatch: Dispatch;
}

export const Preview: React.FC<Props> = ({ tree, headings, path, titleBar, vibrant, dispatch }) => {
    if (tree.root === null) {
        return <></>;
    }

    const sx = vibrant ? {} : SX_NON_VIBRANT;
    return (
        <Box component="main" sx={sx}>
            <Resizable defaultSize={NAV_DEFAULT_SIZE} minWidth="200px" enable={NAV_RESIZE_DIRECTION} as="nav">
                {titleBar && <WindowBar />}
                <SideBar headings={headings} path={path} />
            </Resizable>
            <Divider orientation="vertical" />
            <Article tree={tree} dispatch={dispatch} />
        </Box>
    );
};
