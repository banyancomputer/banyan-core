import React from 'react';
import { FileIcon } from '../FileIcon';

export const FileCell: React.FC<{ name: string }> = ({ name }) => {
    return (
        <div className="flex items-center gap-3 cursor-pointer">
            <FileIcon fileName={name} className="p-2.5 flex bg-button-primary text-secondaryBackground rounded-full" />
            <span className='overflow-hidden text-ellipsis whitespace-nowrap font-medium'>
                {name}
            </span>
        </div>
    )
};
