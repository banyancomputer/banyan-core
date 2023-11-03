import React from 'react';
import { useIntl } from 'react-intl';
import { FiTrash2 } from 'react-icons/fi';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';

export const DeleteFileModal: React.FC<{ bucket: Bucket; file: BrowserObject; path: string[]; parrentFolder: BrowserObject }> = ({ bucket, file, path, parrentFolder }) => {
    const { closeModal } = useModal();
    const { messages } = useIntl();
    const { deleteFile, getSelectedBucketFiles, getExpandedFolderFiles } = useTomb();
    const folderLocation = useFolderLocation();

    const removeFile = async() => {
        try {
            await deleteFile(bucket, [...path], file.name);
            if (path.join('/') === folderLocation.join('/')) {
                await getSelectedBucketFiles(folderLocation);
            } else {
                await getExpandedFolderFiles(path, parrentFolder, bucket);
            };
            closeModal();
            ToastNotifications.notify(`${messages.file} "${file.name}" ${messages.wasDeleted}`, <FiTrash2 size="20px" />);
        } catch (error: any) {
            ToastNotifications.error(`${messages.deletionError}`, `${messages.tryAgain}`, removeFile);
        };
    };

    return (
        <div className="w-modal flex flex-col gap-5">
            <FiTrash2 size="24px" stroke="#5e6c97" />
            <div>
                <h4 className="text-m font-semibold">{`${messages.removeFile}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.wantToMove}`} <b className="text-text-900">{file.name}</b>? <br /> {`${messages.filesWillBeMoved}`}.
                </p>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary flex-grow py-3 px-4"
                    onClick={removeFile}
                >
                    {`${messages.delete}`}
                </button>
            </div>
        </div>
    );
};
