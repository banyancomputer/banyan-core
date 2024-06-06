import React from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useAppDispatch, useAppSelector } from '@/app/store';
import { closeModal } from '@/app/store/modals/slice';
import { deleteFile, getExpandedFolderFiles, getSelectedBucketFiles } from '@/app/store/tomb/actions';

import { Trash } from '@static/images/common';

export const DeleteFileModal: React.FC<{ bucket: Bucket; file: BrowserObject; path: string[]; parrentFolder: BrowserObject }> = ({ bucket, file, path, parrentFolder }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.deleteFile);
    const dispatch = useAppDispatch();
    const folderLocation = useFolderLocation();

    const cancel = () => {
        dispatch(closeModal());
    };

    const removeFile = async () => {
        try {
            unwrapResult(await dispatch(deleteFile({bucket, path: [...path], name: file.name})));
            cancel();
            if (path.join('/') === folderLocation.join('/')) {
                unwrapResult(await dispatch(getSelectedBucketFiles(folderLocation)));
            } else {
                unwrapResult(await dispatch(getExpandedFolderFiles({ path, folder: parrentFolder })));
            };
            dispatch(closeModal());
            ToastNotifications.notify(`${messages.file} "${file.name}" ${messages.wasDeleted}`, <Trash width="20px" height="20px" />);
        } catch (error: any) {
            ToastNotifications.error(`${messages.deletionError}`, `${messages.tryAgain}`, removeFile);
        };
    };

    return (
        <div className="w-modal flex flex-col gap-5">
            <div>
                <h4 className="text-m font-semibold">{`${messages.title}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.wantToMove}`} <b className="text-text-900">{file.name}</b>? <br /> {`${messages.filesWillBeMoved}`}.
                </p>
            </div>
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={cancel}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.delete}`}
                    action={removeFile}
                />
            </div>
        </div>
    );
};
