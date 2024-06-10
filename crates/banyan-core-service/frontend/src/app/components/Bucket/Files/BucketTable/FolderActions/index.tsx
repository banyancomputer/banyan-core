import React from 'react';

import { MoveToModal } from '@components/common/Modal/MoveToModal';
import { RenameFolderModal } from '@components/common/Modal/RenameFolderModal';
import { DeleteFileModal } from '@components/common/Modal/DeleteFileModal';
import { UploadFileModal } from '@components/common/Modal/UploadFileModal';
import { Action } from '@components/Bucket/Files/BucketTable/FileActions';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { openModal } from '@store/modals/slice';
import { useAppDispatch, useAppSelector } from '@store/index';

import { MoveTo, Rename, Trash, Upload } from '@static/images/common';

export const FolderActions: React.FC<{ bucket: Bucket; folder: BrowserObject; parrentFolder: BrowserObject; path: string[] }> = ({ bucket, folder, path, parrentFolder }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.files.bucketTable.folderActions);
    const bucketType = `${bucket.bucketType}_${bucket.storageClass}`;
    const dispatch = useAppDispatch();

    const uploadFile = () => {
        dispatch(
            openModal(
                {
                    content: <UploadFileModal
                        bucket={bucket}
                        folder={folder}
                        path={[...path, folder.name]}
                    />
                }
            )
        );
    };

    const moveTo = () => {
        dispatch(openModal(
            {
                content: <MoveToModal
                    file={folder}
                    bucket={bucket}
                    path={path}
                    parrentFolder={parrentFolder}
                />
            }
        )
        );
    };

    const rename = async () => {
        dispatch(openModal(
            {
                content: <RenameFolderModal
                    bucket={bucket}
                    folder={folder}
                    path={path}
                />
            }
        )
        );
    };

    const remove = async () => {
        dispatch(openModal(
            {
                content: <DeleteFileModal
                    bucket={bucket}
                    file={folder}
                    parrentFolder={parrentFolder}
                    path={path}
                />
            }
        ));
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
        <div className="absolute w-48 right-5 text-xs font-medium bg-bucket-actionsBackground rounded-md shadow-md z-10 select-none text-bucket-actionsText overflow-hidden">{
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
