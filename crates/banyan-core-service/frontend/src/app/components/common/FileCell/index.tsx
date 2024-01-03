import React from 'react';
import { FileIcon } from '../FileIcon';

export const FileCell: React.FC<{ name: string, type: string }> = ({ name, type }) =>
    <div className="flex items-center gap-3 cursor-pointer">
        <FileIcon fileName={name} type={type} />
        <span className="overflow-hidden text-ellipsis whitespace-nowrap font-medium">
            {name}
        </span>
    </div>;

