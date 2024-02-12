import React, { SVGProps } from 'react';

import { CommonFileIcon, Folder, ImageFileIcon, VideoFileIcon, PdfFileIcon, WordFileIcon, AudioFileIcon } from '@static/images/common';

export const FileIcon: React.FC<{ fileName: string; type: string, className?: string; size?: string }> = ({ fileName, className, type, size = '20px' }) => {
    const fileTypeMapper: Record<string, React.FC<SVGProps<any>>> = {
        'txt': CommonFileIcon,
        'pdf': PdfFileIcon,
        'doc': WordFileIcon,
        'docx': WordFileIcon,
        'jpg': ImageFileIcon,
        'jpeg': ImageFileIcon,
        'png': ImageFileIcon,
        'gif': ImageFileIcon,
        'mp4': VideoFileIcon,
        'mkv': VideoFileIcon,
        'webm': VideoFileIcon,
        'mp3': AudioFileIcon,
        'wav': AudioFileIcon,
        'mov': VideoFileIcon,
        'ogg': VideoFileIcon,
        'fig': CommonFileIcon,
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
