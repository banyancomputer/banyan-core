import React from 'react';

import { FileIcon } from '../FileIcon';

import { BrowserObject } from '@app/types/bucket';

import { Folder } from '@/app/static/images/common';

export const FileCell: React.FC<{ borwserObject: BrowserObject }> = ({ borwserObject }) =>
    <div className="flex items-center gap-3 cursor-pointer">
        {borwserObject.type === 'file' ?
            <FileIcon browserObject={borwserObject} />
            :
            <Folder />
        }
        <span className="overflow-hidden text-ellipsis whitespace-nowrap font-medium">
            {borwserObject.name}
        </span>
    </div>;

