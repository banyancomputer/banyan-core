import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { FolderSelect } from '@components/common/FolderSelect';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { useModal } from '@/app/contexts/modals';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useTomb } from '@/app/contexts/tomb';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useFilePreview } from '@/app/contexts/filesPreview';
import { useAppSelector } from '@/app/store';
import { stringToBase64 } from '@/app/utils/base64';

export const MoveToModal: React.FC<{
    file: BrowserObject;
    bucket: Bucket;
    path: string[];
    parrentFolder: BrowserObject,
    createdFolderPath?: string[]
}> = ({ file, bucket, path, parrentFolder, createdFolderPath }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.moteTo);
    const { moveTo, getSelectedBucketFiles, getExpandedFolderFiles } = useTomb();
    const navigate = useNavigate();
    const { closeModal, openModal } = useModal();
    const { closeFile } = useFilePreview();
    const [selectedFolder, setSelectedFolder] = useState<string[]>([]);
    const folderLocation = useFolderLocation();

    const move = async () => {
        try {
            await moveTo(bucket, [...path, file.name], [...selectedFolder], file.name);
            closeFile();
            ToastNotifications.notify(
                `${file.type === 'dir' ? messages.fileWasMoved : messages.fileWasMoved}`,
                null,
                file.type === 'dir' ? messages.viewFolder : messages.viewFile,
                () => navigate(`/drive/${bucket.id}${selectedFolder.length ? '?' : ''}${selectedFolder.map(path => stringToBase64(path)).join('/')}`)
            );
            if (path.join('/') === folderLocation.join('/')) {
                await getSelectedBucketFiles(folderLocation);
                closeModal();

                return;
            };
            await getExpandedFolderFiles(path, parrentFolder, bucket);
            closeModal();
        } catch (error: any) {
            ToastNotifications.error(`${messages.moveToError}`, `${messages.tryAgain}`, move);
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
                            openModal(
                                <MoveToModal
                                    bucket={bucket}
                                    file={file}
                                    path={path}
                                    parrentFolder={parrentFolder}
                                    createdFolderPath={createdFolderPath}
                                />
                            )
                    }
                />
            </div>
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
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
