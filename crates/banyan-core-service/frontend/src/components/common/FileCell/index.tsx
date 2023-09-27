import React from 'react';
import { FileIcon } from '../FileIcon';

export const FileCell: React.FC<{ name: string }> = ({ name }) => {
    return (
        <div className="flex items-center gap-3 cursor-pointer">
            <FileIcon fileName={name} className="p-2 bg-gray-200 rounded-full" />
            <span className='overflow-hidden text-ellipsis whitespace-nowrap'>
                {name}
            </span>
        </div>
    )
};
