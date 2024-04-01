import React from 'react';

import { useFolderLocation } from '@app/hooks/useFolderLocation';
import { Bucket } from '@app/types/bucket';

import { Upload } from '@static/images/common';
import { useFilesUpload } from '@contexts/filesUpload';
import { ToastNotifications } from '@utils/toastNotifications';
import { useAppSelector } from '@/app/store';

export const EmptyState: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.files.emptyState);
    const { uploadFiles } = useFilesUpload();

    const folderLocation = useFolderLocation();

    const handleDrop = async (event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();

        if (!event.dataTransfer.files) { return; }

        try {
            await uploadFiles(event.dataTransfer.files, bucket!, folderLocation);
        } catch (error: any) {
            ToastNotifications.error(messages.uploadError);
        };
    };

    const handleChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
        if (!event.target.files) { return; }

        try {
            await uploadFiles(event.target.files, bucket!, folderLocation);
        } catch (error: any) {
            ToastNotifications.error(messages.uploadError);
        };
    };

    const handleDrag = async (event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();
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
                <Upload width="63" height="63px" />
                <div className="mt-14 flex flex-col items-center">
                    <p className="text-text-900">
                        {messages.description}
                    </p>
                </div>
                <span
                    className="btn-secondary mt-4 flex items-center gap-2 px-4 py-2 text-xs font-semibold rounded-md bg-text-200 text-button-primary"
                >
                    <Upload />
                    {messages.buttonText}
                </span>
            </div>
        </label>
    )
}
