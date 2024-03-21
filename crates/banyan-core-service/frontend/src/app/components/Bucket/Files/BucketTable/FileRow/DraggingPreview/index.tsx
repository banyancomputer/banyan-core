import React from 'react';

import { FileIcon } from '@components/common/FileIcon';

export const DraggingPreview: React.FC<{ name: string; isDragging: boolean, type: string }> = ({ name, isDragging, type }) =>
    <>
        {isDragging ?
            <div
                className="fixed flex items-center gap-3 p-2 border-1 text-xs leading-3 bg-secondaryBackground border-border-darken rounded-xl z-max pointer-events-none"
                id={`dragging-preview-${name}`}
            >
                <FileIcon fileName={name} type={type} size="24px" />
                {name}
            </div>
            :
            null
        }
    </>;

