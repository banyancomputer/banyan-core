import React from 'react';
import { useIntl } from 'react-intl';

import { useFolderLocation } from '@app/hooks/useFolderLocation';
import { Bucket } from '@app/types/bucket';

import { Upload } from '@static/images/common';
import { useFilesUpload } from '@app/contexts/filesUpload';
import { ToastNotifications } from '@utils/toastNotifications';

export const EmptyState: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { messages } = useIntl();
    const { setFiles, uploadFiles, files } = useFilesUpload();

    const folderLocation = useFolderLocation();

    const handleDrop = async (event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();

        if (!event.dataTransfer.files) { return; }

        setFiles(Array.from(event.dataTransfer.files).slice(0, 1).map(file => ({ file, status: 'pending' })));
    };

    const handleChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
        if (!event.target.files) { return; }

        setFiles(Array.from(event.target.files).map(file => ({ file, status: 'pending' })));
    };

    const handleDrag = async (event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();
    };

    const upload = async () => {
        if (!files.length) { return; }
        try {
            ToastNotifications.uploadProgress();
            await uploadFiles(bucket!, folderLocation);
        } catch (error: any) {
            ToastNotifications.error(`${messages.uploadError}`, `${messages.tryAgain}`, upload);
        };
    };

    return (
        <label
            className="flex-grow flex justify-center items-center border-1 border-border-darken border-dashed cursor-pointer"
            onDrop={handleDrop}
            onDragOver={handleDrag}
        >
            <input
                type="file"
                multiple={false}
                className="hidden"
                onChange={handleChange}
            />
            <div
                className="flex flex-col items-center text-[#A99996]"
            >
                {!files.length ?
                    <>
                        <Upload width="63" height="63px" />
                        <div className="mt-14 flex flex-col items-center">
                            <p className="text-text-900">
                                Drag & drop files here to upload,or use the 'Upload' button
                            </p>
                        </div>
                    </>
                    :
                    <>
                        {files.map(file =>
                            <span
                                className="overflow-hidden text-ellipsis whitespace-nowrap text-text-900"
                                key={file.file.name}
                            >
                                {file.file.name}
                            </span>
                        )}
                        <button
                            className="mt-4 flex items-center gap-2 px-4 py-2 text-xs font-semibold rounded-md bg-text-200 text-button-primary"
                            onClick={upload}
                        >
                            <Upload />
                            {`${messages.upload}`}
                        </button>
                    </>
                }
            </div>
        </label>
    )
}
