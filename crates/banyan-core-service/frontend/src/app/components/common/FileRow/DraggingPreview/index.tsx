import React from 'react';

import { FileIcon } from '@components/common/FileIcon';

export const DraggingPreview: React.FC<{ name: string }> = ({ name }) => {
    return (
        <div className='flex items-start  gap-3 p-2 border-1 text-xs leading-3 border-border-darken'>
            <FileIcon fileName={name} size='24px' />
            {name}
        </div>
    )
};
