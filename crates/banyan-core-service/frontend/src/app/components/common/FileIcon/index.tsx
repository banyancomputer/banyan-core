import React, { useEffect, useState } from 'react';

import { BrowserObject } from '@app/types/bucket';
import { SUPPORTED_FILE_TYPES } from '@app/types/filesPreview';

import { CommonFileIcon } from '@static/images/common';

export const FileIcon: React.FC<{ browserObject: BrowserObject, className?: string; size?: string }> = ({ browserObject, className, size = '20px' }) => {
    const [icon, setIcon] = useState(<CommonFileIcon width={size} height={size} />);

    useEffect(() => {
        SUPPORTED_FILE_TYPES.some(element => {
            const result = element.mimeTypes.includes(browserObject.metadata.mime || '');
            if (result) {
                const Icon = element.icon;
                setIcon(<Icon width={size} height={size} />);
                return;
            }
        });
    }, []);

    return (
        <div className={className}>
            {icon}
        </div>
    );
};
