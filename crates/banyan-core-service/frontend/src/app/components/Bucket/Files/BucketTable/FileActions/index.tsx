import React, { ReactElement, useMemo } from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { MoveToModal } from '@components/common/Modal/MoveToModal';
import { RenameFileModal } from '@components/common/Modal/RenameFileModal';
import { ShareFileModal } from '@components/common/Modal/ShareFileModal';
import { DeleteFileModal } from '@components/common/Modal/DeleteFileModal';

import { openModal } from '@store/modals/slice';
import { Copy, Done, Download, LinkIcon, MoveTo, Rename, Share, Trash } from '@static/images/common';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppDispatch, useAppSelector } from '@store/index';

import { getFile, shareFile, uploadFile } from '@store/tomb/actions';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';

export class Action {
    constructor(
        public label: string,
        public icon: ReactElement,
        public value: () => void,
        public tooltip?: string
    ) { }
}

export const FileActions: React.FC<{ bucket: Bucket; file: BrowserObject; parrentFolder: BrowserObject; path: string[] }> = ({ bucket, file, path, parrentFolder }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.files.bucketTable.fileActions);
    const dispatch = useAppDispatch();
    const bucketType = `${bucket.bucketType}_${bucket.storageClass}`;
    const folderLocation = useFolderLocation();


    const downloadFile = async() => {
        try {
            await ToastNotifications.promise(`${messages.downloading}...`, `${messages.fileWasDownloaded}`, <Done width="20px" height="20px" />,
                (async () => {
                    const link = document.createElement('a');
                    const arrayBuffer = unwrapResult(await dispatch(getFile({ bucket: bucket!, path, name: file.name })));
                    const blob = new Blob([arrayBuffer]);
                    const objectURL = URL.createObjectURL(blob);
                    link.href = objectURL;
                    link.download = file.name;
                    document.body.appendChild(link);
                    link.click();
                })()
            );
        } catch (error: any) {
            ToastNotifications.error('Failed to download file', messages.tryAgain, downloadFile);
        }
    };

    const moveTo = () => {
        dispatch(openModal(
            {
                content: <MoveToModal
                    file={file}
                    bucket={bucket}
                    path={path}
                    parrentFolder={parrentFolder}
                />,
            }
        ));
    };

    const copy = async() => {
        try {
            const arrayBuffer: ArrayBuffer = unwrapResult(await dispatch(getFile({ bucket, path, name: file.name })));
            await dispatch(uploadFile({ bucket, uploadPath: path, name: `Copy of ${file.name}`, file: arrayBuffer, folderLocation }));
            ToastNotifications.notify(`${messages.copyOf} ${file.name} ${messages.wasCreated}`);
        } catch (error: any) {
            ToastNotifications.error('Error while copying file', `${messages.tryAgain}`, copy);
        };
    };

    const rename = async() => {
        dispatch(openModal({
            content: <RenameFileModal
                bucket={bucket}
                file={file}
                path={path}
            />,
        }));
    };

    const remove = async() => {
        try {
            dispatch(openModal({
                content: <DeleteFileModal
                    bucket={bucket}
                    file={file}
                    parrentFolder={parrentFolder}
                    path={path}
                />,
            }));
        } catch (error: any) { }
    };

    const viewFileVersions = async() => {
        try {

        } catch (error: any) { }
    };

    const share = async() => {
        try {
            const payload = unwrapResult(await dispatch(shareFile({ bucket, path: [...path, file.name] })));
            const link = `${window.location.origin}/api/v1/share?payload=${payload}`;
            dispatch(openModal({
                content: <ShareFileModal link={link} />,
            }));
        } catch (error: any) {
            ToastNotifications.error('Error while sharing file', `${messages.tryAgain}`, share);
        }
    };

    const downloadAction = useMemo(() => new Action(messages.download, <Download width="18px" height="18px" />, downloadFile), []);
    const moveToAction = new Action(messages.moveTo, <MoveTo width="18px" height="18px" />, moveTo);
    const makeCopyAction = new Action(messages.makeCopy, <Copy width="18px" height="18px" />, copy);
    const vierFileVersionsAction = new Action(messages.viewFileVersions, <LinkIcon width="18px" height="18px" />, viewFileVersions);
    const renameAction = new Action(messages.rename, <Rename width="18px" height="18px" />, rename);
    const removeAction = new Action(messages.remove, <Trash width="18px" height="18px" />, remove);
    const shareAction = new Action(messages.shareFile, <Share width="18px" height="18px" />, share);

    const hotInrecactiveActions = [
        downloadAction, moveToAction, makeCopyAction, renameAction, removeAction,
    ];
    const warmInrecactiveActions = [
        downloadAction, moveToAction, makeCopyAction, renameAction, removeAction,
    ];
    const coldIntecactiveActions = [
        downloadAction,
    ];
    const hotBackupActions = [
        downloadAction, makeCopyAction,
    ];
    const warmBackupActions = [
        downloadAction,
    ];
    const coldBackupActions = [
        downloadAction,
    ];

    const actions: Record<string, Action[]> = {
        interactive_hot: hotInrecactiveActions,
        interactive_warm: warmInrecactiveActions,
        interactive_cold: coldIntecactiveActions,
        backup_hot: hotBackupActions,
        backup_warm: warmBackupActions,
        backup_cold: coldBackupActions,
    };

    return (
        <div className="absolute w-48 right-5 text-xs font-medium bg-bucket-actionsBackground rounded-md shadow-md z-10 text-bucket-actionsText select-none">{
            actions[bucketType].map(action =>
                <div
                    key={action.label}
                    className="w-full flex items-center gap-2 py-2 px-3 transition-all hover:bg-hover"
                    onClick={action.value}
                >
                    {action.icon}
                    {action.label}
                </div>
            )
        }
        <div
            className="w-full flex justify-between items-center gap-2 py-2 px-3 border-t-1 border-border-regular transition-all hover:bg-hover"
        >
                Your file is secure <span className="rounded-full w-2 h-2" style={{ background: '#2bb65e' }} />
        </div>
        </div>
    );
};
