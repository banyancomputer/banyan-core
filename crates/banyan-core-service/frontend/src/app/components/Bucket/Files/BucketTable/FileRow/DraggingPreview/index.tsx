import React from 'react';

import { FileIcon } from '@components/common/FileIcon';
import { BrowserObject } from '@/app/types/bucket';

export const DraggingPreview: React.FC<{ browserObject: BrowserObject, isDragging: boolean }> = ({ browserObject, isDragging }) =>
    <>
        {isDragging ?
            <div
                className="fixed flex items-center gap-3 p-2 border-1 text-xs leading-3 bg-secondaryBackground border-border-darken rounded-xl z-max pointer-events-none"
                id={`dragging-preview-${browserObject.name}`}
            >
                <FileIcon browserObject={browserObject} size="24px" />
                {browserObject.name}
            </div>
            :
            null
        }
    </>;

