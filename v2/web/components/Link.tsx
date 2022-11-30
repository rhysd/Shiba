import React from 'react';
import { Tooltip } from '@mui/material';

const TIP_STYLE: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    margin: 4,
    fontSize: '0.8rem',
};

export interface Props {
    title: string | undefined;
    href: string;
    children: React.ReactNode[];
}

export const Link: React.FC<Props> = ({ href, title, children }) => {
    const tipTitle = (
        <div style={TIP_STYLE}>
            {title}
            <div>{href}</div>
        </div>
    );
    return (
        <Tooltip title={tipTitle} arrow disableInteractive>
            <a href={href}>{children}</a>
        </Tooltip>
    );
};
