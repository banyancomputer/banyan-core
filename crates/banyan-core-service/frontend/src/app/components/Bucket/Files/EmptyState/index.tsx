import React from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { useFolderLocation } from '@app/hooks/useFolderLocation';
import { Bucket } from '@app/types/bucket';

import { Upload } from '@static/images/common';
import { ToastNotifications } from '@utils/toastNotifications';
import { useAppDispatch, useAppSelector } from '@store/index';
import { uploadFiles } from '@store/filesUpload/actions';

export const EmptyState: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.files.emptyState);
    const dispatch = useAppDispatch();

    const folderLocation = useFolderLocation();

    const handleDrop = async (event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();

        if (!event.dataTransfer.files) { return; }

        try {
            unwrapResult(await dispatch(uploadFiles({ fileList: event.dataTransfer.files, bucket: bucket!, path: folderLocation, folderLocation })));
        } catch (error: any) {
            ToastNotifications.error(messages.uploadError);
        };
    };

    const handleChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
        if (!event.target.files) { return; }

        try {
            unwrapResult(await dispatch(uploadFiles({ fileList: event.target.files, bucket: bucket!, path: folderLocation, folderLocation })));
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
                multiple
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
