import React from 'react';
import { useIntl } from 'react-intl';

import { Loader } from '../Loader';
import { FileIcon } from '../FileIcon';

import { useFilesUpload } from '@/app/contexts/filesUpload';
import { Done } from '@static/images/common';

export const UploadFileProgress = () => {
    const { messages } = useIntl();
    const { files } = useFilesUpload();

    return (
        <div className="w-80">
            <div className="flex justify-between items-center px-3 py-2 bg-button-primary text-white font-normal text-xs">
                <p>{`${messages.uploading}`}</p>
            </div>
            <div className="flex flex-col">
                {files.map(file =>
                    <div
                        className="flex items-center px-3 py-2 gap-2 text-xs font-normal text-bucket-actionsText"
                        key={file.file.name}
                    >
                        <FileIcon fileName={file.file.name} type="file" />
                        <span className="flex-grow text-bucket-actionsText">{file.file.name}</span>
                        <span className="w-5 h-5">
                            {file.isUploaded ?
                                <span className="flex items-center justify-center p-1 bg-gray-800 text-white rounded-full">
                                    <Done width="12px" height="12px" />
                                </span>
                                :
                                <Loader spinnerSize="20px" containerHeight="100%" />
                            }
                        </span>
                    </div>
                )}
            </div>
        </div>
    );
};
