import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { unwrapResult } from '@reduxjs/toolkit';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { FolderSelect } from '@components/common/FolderSelect';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { closeModal, openModal } from '@store/modals/slice';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { stringToBase64 } from '@/app/utils/base64';
import { useAppDispatch, useAppSelector } from '@store/index';
import { getExpandedFolderFiles, getSelectedBucketFiles, moveTo } from '@store/tomb/actions';
import { closeFile } from '@store/filePreview/slice';

export const MoveToModal: React.FC<{
    file: BrowserObject;
    bucket: Bucket;
    path: string[];
    parrentFolder: BrowserObject,
    createdFolderPath?: string[]
}> = ({ file, bucket, path, parrentFolder, createdFolderPath }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.moteTo);
    const navigate = useNavigate();
    const [selectedFolder, setSelectedFolder] = useState<string[]>([]);
    const folderLocation = useFolderLocation();
    const dispatch = useAppDispatch();

    const close = () => {
        dispatch(closeModal());
    };

    const move = async () => {
        try {
            unwrapResult(await dispatch(moveTo({ bucket, from: [...path, file.name], to: [...selectedFolder], name: file.name })));
            dispatch(closeFile());
            ToastNotifications.notify(
                `${file.type === 'dir' ? messages.fileWasMoved : messages.fileWasMoved}`,
                null,
                file.type === 'dir' ? messages.viewFolder : messages.viewFile,
                () => navigate(`/drive/${bucket.id}${selectedFolder.length ? '?' : ''}${selectedFolder.map(path => stringToBase64(path)).join('/')}`)
            );
            if (path.join('/') === folderLocation.join('/')) {
                unwrapResult(await dispatch(getSelectedBucketFiles(folderLocation)));
                close();

                return;
            };
            unwrapResult(await dispatch(getExpandedFolderFiles({ path, folder: parrentFolder })));
            close();
        } catch (error: any) {
            ToastNotifications.error(`${messages.moveToError}`, `${messages.tryAgain}`, move);
            close();
        };
    };

    const selectFolder = (option: string[]) => {
        setSelectedFolder(option);
    };

    useEffect(() => {
        if (!createdFolderPath) return;

        setSelectedFolder(createdFolderPath);
    }, [createdFolderPath]);

    return (
        <div className="w-modal flex flex-col gap-6" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.subtitle}`}
                </p>
            </div>
            <div>
                <label className="inline-block mb-1 text-xs font-normal">{`${messages.folder}`}:</label>
                <FolderSelect
                    selectedFolder={selectedFolder}
                    selectedBucket={bucket}
                    onChange={selectFolder}
                    onFolderCreation={
                        (createdFolderPath?: string[]) =>
                            dispatch(openModal({
                                content: <MoveToModal
                                    bucket={bucket}
                                    file={file}
                                    path={path}
                                    parrentFolder={parrentFolder}
                                    createdFolderPath={createdFolderPath}
                                />
                            }))
                    }
                />
            </div>
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={close}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.moveTo}`}
                    action={move}
                    disabled={path.join('/') === selectedFolder.join('/')}
                />
            </div>
        </div>
    );
};
