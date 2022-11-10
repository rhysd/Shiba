import React from 'react';
import { Tooltip } from '@mui/material';

// This component will be used as drop-in replacement of <a> in rehype-react
type Props = React.AnchorHTMLAttributes<HTMLAnchorElement>;

const TIP_STYLE: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    margin: 4,
    fontSize: '0.8rem',
};

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
