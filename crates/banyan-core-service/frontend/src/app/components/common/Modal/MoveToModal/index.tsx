import React, { useState } from 'react';

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

export const MoveToModal: React.FC<{ file: BrowserObject; bucket: Bucket; path: string[]; parrentFolder: BrowserObject }> = ({ file, bucket, path, parrentFolder }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.moteTo);
    const { moveTo, getSelectedBucketFiles, getExpandedFolderFiles } = useTomb();
    const { closeModal, openModal } = useModal();
    const { closeFile } = useFilePreview();
    const [selectedFolder, setSelectedFolder] = useState<string[]>([]);
    const folderLocation = useFolderLocation();

    const move = async () => {
        try {
            await moveTo(bucket, [...path, file.name], [...selectedFolder], file.name);
            closeFile();
            ToastNotifications.notify(`${messages.fileWasMoved}`);
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
                    selectedBucket={bucket}
                    onChange={selectFolder}
                    path={path}
                    onFolderCreation={() => openModal(<MoveToModal bucket={bucket} file={file} path={path} parrentFolder={parrentFolder} />)}
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
