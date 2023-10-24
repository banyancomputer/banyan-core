import React, { useState } from 'react';
import { useIntl } from 'react-intl';
import { MdDone } from 'react-icons/md';

import { FolderSelect } from '../../FolderSelect';
import { useModal } from '@/contexts/modals';
import { BrowserObject, Bucket } from '@/lib/interfaces/bucket';
import { ToastNotifications } from '@/utils/toastNotifications';
import { useTomb } from '@/contexts/tomb';

import { useFolderLocation } from '@/hooks/useFolderLocation';

export const MoveToModal: React.FC<{ file: BrowserObject; bucket: Bucket; path: string[]; parrentFolder: BrowserObject }> = ({ file, bucket, path, parrentFolder }) => {
    const { messages } = useIntl();
    const { moveTo, getSelectedBucketFiles, getExpandedFolderFiles } = useTomb();
    const { closeModal, openModal } = useModal();
    const [selectedFolder, setSelectedFolder] = useState<string[]>([]);
    const folderLocation = useFolderLocation();

    const move = async() => {
        try {
            await moveTo(bucket, [...path, file.name], [...selectedFolder, file.name]);
            ToastNotifications.notify(`${messages.fileWasMoved}`, <MdDone size="20px" />);
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
                <h4 className="text-m font-semibold ">{`${messages.moveTo}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.selectWhereToMove}`}
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
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary flex-grow py-3 px-4"
                    onClick={move}
                >
                    {`${messages.moveTo}`}
                </button>
            </div>
        </div>
    );
};
