import * as React from 'react';
import IconButton from '@mui/material/IconButton';
import MoreVertIcon from '@mui/icons-material/MoreVert';
import { sendMessage } from '../ipc';

function onClick(e: React.MouseEvent<HTMLElement>): void {
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    const x = rect.x + rect.width + 4.0;
    const y = rect.y + rect.height + 4.0;
    sendMessage({ kind: 'open_menu', position: [x, y] });
}

interface Props {
    style?: React.CSSProperties;
}

export const MenuButton: React.FC<Props> = ({ style = {} }) => {
    return (
        <IconButton id="shiba-menu-button" onClick={onClick} size="small" style={style}>
            <MoreVertIcon fontSize="inherit" />
        </IconButton>
    );
};
