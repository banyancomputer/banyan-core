import { useNavigate } from 'react-router-dom';
import React, { useEffect, useMemo, useState } from 'react';

import { Loader } from '../Loader';

import { useAppDispatch, useAppSelector } from '@store/index';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { stringToBase64 } from '@/app/utils/base64';
import { deleteFile, setFiles } from '@store/filesUpload/slice';
import { retryUpload } from '@store/filesUpload/actions';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';

import { ChevronUp, Clock, Close, Retry, UploadFailIcon, UploadFileFolder, UploadSuccessIcon } from '@static/images/common';

export const UploadFileProgress: React.FC<{ bucket: Bucket, path: string[], folder?: BrowserObject }> = ({ bucket, path, folder }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.uploadFileProgress);
    const { files } = useAppSelector(state => state.filesUpload);
    const [isExpanded, setIsExpanded] = useState(true);
    const uploadedFilesLength = useMemo(() => files.filter(file => file.status === 'success').length, [files]);
    const navigate = useNavigate();
    const folderLocation = useFolderLocation();
    const dispatch = useAppDispatch();

    const toggleVisibility = (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
        event.stopPropagation();
        setIsExpanded(prev => !prev);
    };

    const close = (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
        event.stopPropagation();
        ToastNotifications.close();
        dispatch(setFiles([]));
    };

    const goToFile = () => {
        navigate(`/drive/${bucket.id}${path.length ? '?' : ''}${path.map(path => stringToBase64(path)).join('/')}`);
    };

    const ICONS_MAPPER = {
        pending: <Clock width="20px" height="20px" />,
        success: <UploadSuccessIcon width="20px" height="20px" />,
        failed: <UploadFailIcon width="20px" height="20px" />,
        uploading: <Loader spinnerSize="20px" containerHeight="100%" />
    };

    useEffect(() => {
        if (files.every(file => file.status === 'success')) {
            setTimeout(() => {
                ToastNotifications.close();
                setFiles([]);
            }, 3000);
        };
    }, [files])

    return (
        <div
            className="w-80"
            onClick={toggleVisibility}
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
                <div className={`${isExpanded ? '' : 'rotate-180'}`}>
                    {files.every(file => file.status === 'success') ?
                        <div onClick={close}>
                            <Close />
                        </div>
                        :
                        <ChevronUp />
                    }
                </div>
            </div>
            {isExpanded ?
                <div className="flex flex-col bg-mainBackground">
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
                                    <span className=" text-[#CB3535] whitespace-nowrap">
                                        {messages.uploadFailed}
                                    </span>
                                    <span
                                        className="cursor-pointer"
                                        onClick={() => dispatch(retryUpload({ file, bucket, path, folder, folderLocation }))}
                                    >
                                        <Retry />
                                    </span>
                                    <span
                                        className="text-text-600 cursor-pointer"
                                        onClick={() => dispatch(deleteFile(file))}
                                    >
                                        <Close />
                                    </span>
                                </span>
                            }
                            {file.status === 'success' &&
                                <span onClick={goToFile} className="cursor-pointer">
                                    <UploadFileFolder />
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
