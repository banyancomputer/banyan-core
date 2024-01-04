import React, { SVGProps } from 'react';

import { CommonFileIcon, FigmaFileIcon, Folder, ImageFileIcon, VideoFileIcon, PdfFileIcon } from '@static/images/common';

export const FileIcon: React.FC<{ fileName: string; type: string, className?: string; size?: string }> = ({ fileName, className, type, size = '20px' }) => {
    const fileTypeMapper: Record<string, React.FC<SVGProps<any>>> = {
        'txt': CommonFileIcon,
        'pdf': PdfFileIcon,
        'doc': CommonFileIcon,
        'docx': CommonFileIcon,
        'jpg': ImageFileIcon,
        'jpeg': ImageFileIcon,
        'png': ImageFileIcon,
        'gif': ImageFileIcon,
        'mp4': VideoFileIcon,
        'mov': VideoFileIcon,
        'mkv': VideoFileIcon,
        'webm': VideoFileIcon,
        'fig': FigmaFileIcon,
    };

    const Icon = fileTypeMapper[fileName.split('.').pop() || ''];

    return (
        <div className={className}>
            {
                type === 'file' ?
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
