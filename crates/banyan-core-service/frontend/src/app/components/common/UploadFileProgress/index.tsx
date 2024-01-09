import React, { useEffect, useMemo, useState } from 'react';
import { useIntl } from 'react-intl';

import { Loader } from '../Loader';

import { useFilesUpload } from '@/app/contexts/filesUpload';

import { ChevronUp, Clock, Close, Retry, UploadFailIcon, UploadSuccessIcon } from '@static/images/common';

export const UploadFileProgress = () => {
    const { messages } = useIntl();
    const { files, deleteFromUploadList, retryUpload } = useFilesUpload();
    const [isExpanded, setIsExpanded] = useState(false);
    const uploadedFilesLength = useMemo(() => files.filter(file => file.status === 'success').length, [files]);

    const toggleVisibility = () => {
        setIsExpanded(prev => !prev);
    };

    const ICONS_MAPPER = {
        pending: <Clock width="20px" height="20px" />,
        success: <UploadSuccessIcon width="20px" height="20px" />,
        failed: <UploadFailIcon width="20px" height="20px" />,
        uploading: <Loader spinnerSize="20px" containerHeight="100%" />
    };

    return (
        <div
            className="w-80"
            onClick={event => event.stopPropagation()}
        >
            <div className="flex justify-between items-center p-4 bg-navigation-primary text-text-900 font-semibold text-xs">
                <div className="flex items-center gap-3">
                    <p>{`${messages.uploading}`}</p>
                    {!isExpanded &&
                        <>
                            {`${uploadedFilesLength} of ${files.length}`}
                        </>
                    }
                </div>
                <div
                    className={`${isExpanded ? '' : 'rotate-180'}`}
                    onClick={toggleVisibility}
                >
                    <ChevronUp />
                </div>
            </div>
            {isExpanded ?
                <div className="flex flex-col">
                    {files.map(file =>
                        <div
                            className="flex items-center px-3 py-2 gap-3 text-xs font-normal text-bucket-actionsText"
                            key={file.file.name}
                        >
                            <span className="w-5 h-5 min-w-[20px]">
                                {ICONS_MAPPER[file.status]}
                            </span>
                            <span
                                className="flex-grow text-bucket-actionsText whitespace-nowrap overflow-hidden text-ellipsis font-semibold"
                            >
                                {file.file.name}
                            </span>
                            {file.status === 'failed' &&
                                <span className="flex items-center gap-3">
                                    <span
                                        className="cursor-pointer"
                                        onClick={() => retryUpload(file)}
                                    >
                                        <Retry />
                                    </span>
                                    <span
                                        className="text-text-600 cursor-pointer"
                                        onClick={() => deleteFromUploadList(file)}
                                    >
                                        <Close />
                                    </span>
                                </span>
                            }
                        </div>
                    )}
                </div>
                :
                null
            }
        </div>
    );
};
