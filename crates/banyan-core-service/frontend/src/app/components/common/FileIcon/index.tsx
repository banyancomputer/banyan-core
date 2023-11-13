import React, { SVGProps } from 'react';

import { Folder, CommonFileIcon, ImageFileIcon, TextFileIcon, VideoFileIcon } from '@static/images/common';

export const FileIcon: React.FC<{ fileName: string; className?: string; size?: string }> = ({ fileName, className, size = '20px' }) => {
    const fileTypeMapper: Record<string, React.FC<SVGProps<any>>> = {
        'txt': TextFileIcon,
        'pdf': TextFileIcon,
        'doc': TextFileIcon,
        'docx': TextFileIcon,
        'jpg': ImageFileIcon,
        'jpeg': ImageFileIcon,
        'png': ImageFileIcon,
        'gif': ImageFileIcon,
        'mp4': VideoFileIcon,
        'mov': VideoFileIcon,
        'mkv': VideoFileIcon,
    };

    const Icon = fileTypeMapper[fileName.split('.')[1]];

    return (
        <div className={className}>
            {
                fileName.split('.')[1] ?
                    Icon ?
                        <Icon width={size} height={size} />
                        :
                        <CommonFileIcon width={size} height={size} />
                    :
                    <Folder width={size} height={size} />
            }
        </div>
    );
};
