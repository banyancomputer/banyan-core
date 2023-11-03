import React from 'react';
import { useIntl } from 'react-intl';
import { MdDone } from 'react-icons/md';

import { Loader } from '../Loader';
import { FileIcon } from '../FileIcon';

import { useFilesUpload } from '@/app/contexts/filesUpload';

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
                        className="flex items-center px-3 py-2 gap-2 text-xs font-normal text-text-800"
                        key={file.file.name}
                    >
                        <FileIcon size="20px" fileName={file.file.name} />
                        <span className="flex-grow text-text-900">{file.file.name}</span>
                        <span className="w-5 h-5">
                            {file.isUploaded ?
                                <span className="flex items-center justify-center p-1 bg-gray-800 text-white rounded-full">
                                    <MdDone size="12px" />
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
