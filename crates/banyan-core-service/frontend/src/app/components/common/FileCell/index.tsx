import React from 'react';
import { FileIcon } from '../FileIcon';

export const FileCell: React.FC<{ name: string }> = ({ name }) =>
    <div className="flex items-center gap-3 cursor-pointer">
        <FileIcon fileName={name} />
        <span className="overflow-hidden text-ellipsis whitespace-nowrap font-medium">
            {name}
        </span>
    </div>;

