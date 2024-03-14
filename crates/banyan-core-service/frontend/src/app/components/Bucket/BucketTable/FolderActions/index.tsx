import React from 'react';

import { MoveToModal } from '@components/common/Modal/MoveToModal';
import { RenameFileModal } from '@components/common/Modal/RenameFileModal';
import { DeleteFileModal } from '@components/common/Modal/DeleteFileModal';
import { UploadFileModal } from '@components/common/Modal/UploadFileModal';
import { Action } from '@components/Bucket/BucketTable/FileActions';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';
import { useAppSelector } from '@/app/store';

import { MoveTo, Rename, Trash, Upload } from '@static/images/common';

export const FolderActions: React.FC<{ bucket: Bucket; file: BrowserObject; parrentFolder: BrowserObject; path: string[] }> = ({ bucket, file, path, parrentFolder }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.bucketTable.folderActions);
    const { openModal } = useModal();
    const bucketType = `${bucket.bucketType}_${bucket.storageClass}`;

    const uploadFile = () => {
        openModal(
            <UploadFileModal
                bucket={bucket}
                folder={file}
                path={[...path, file.name]}
            />
        );
    };

    const moveTo = () => {
        openModal(
            <MoveToModal
                file={file}
                bucket={bucket}
                path={path}
                parrentFolder={parrentFolder}
            />
        );
    };

    const rename = async () => {
        openModal(
            <RenameFileModal
                bucket={bucket}
                file={file}
                path={path}
            />
        );
    };

    const remove = async () => {
        openModal(
            <DeleteFileModal
                bucket={bucket}
                file={file}
                parrentFolder={parrentFolder}
                path={path}
            />
        );
    };

    const moveToAction = new Action(messages.moveTo, <MoveTo width="18px" height="18px" />, moveTo);
    const renameAction = new Action(messages.rename, <Rename width="18px" height="18px" />, rename);
    const removeAction = new Action(messages.remove, <Trash width="18px" height="18px" />, remove);
    const uploadFolderAction = new Action(messages.upload, <Upload width="18px" height="18px" />, uploadFile);

    const hotInrecactiveActions = [
        uploadFolderAction, moveToAction, renameAction, removeAction,
    ];
    const warmInrecactiveActions = [
        uploadFolderAction, moveToAction, renameAction, removeAction,
    ];
    const coldIntecactiveActions = [
        moveToAction,
    ];

    const actions: Record<string, Action[]> = {
        interactive_hot: hotInrecactiveActions,
        interactive_warm: warmInrecactiveActions,
        interactive_cold: coldIntecactiveActions,
        backup_hot: [],
        backup_warm: [],
        backup_cold: [],
    };

    return (
        <div className="w-48 right-8 text-xs font-medium bg-bucket-actionsBackground rounded-md shadow-md z-10 select-none text-bucket-actionsText overflow-hidden">{
            actions[bucketType].map(action =>
                <div
                    key={action.label}
                    className="w-full flex items-center gap-2 py-2 px-3 transition-all hover:bg-hover"
                    onClick={action.value}
                    id="action"
                >
                    {action.icon}
                    {action.label}
                </div>
            )
        }
        </div>
    );
};
