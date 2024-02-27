import React, { ReactElement, useMemo } from 'react';

import { MoveToModal } from '@components/common/Modal/MoveToModal';
import { RenameFileModal } from '@components/common/Modal/RenameFileModal';
import { ShareFileModal } from '@components/common/Modal/ShareFileModal';
import { DeleteFileModal } from '@components/common/Modal/DeleteFileModal';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useTomb } from '@/app/contexts/tomb';
import { useAppSelector } from '@/app/store';

import { Copy, Done, Download, LinkIcon, MoveTo, Rename, Share, Trash } from '@static/images/common';

export class Action {
    constructor(
        public label: string,
        public icon: ReactElement,
        public value: () => void,
        public tooltip?: string
    ) { }
}

export const FileActions: React.FC<{ bucket: Bucket; file: BrowserObject; parrentFolder: BrowserObject; path: string[] }> = ({ bucket, file, path, parrentFolder }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.bucketTable.fileActions);
    const { download, makeCopy, shareFile } = useTomb();
    const { openModal } = useModal();
    const bucketType = `${bucket.bucketType}_${bucket.storageClass}`;

    const downloadFile = async () => {
        try {
            await ToastNotifications.promise(`${messages.downloading}...`, `${messages.fileWasDownloaded}`, <Done width="20px" height="20px" />,
                download(bucket, path, file.name)
            );
        } catch (error: any) {
            ToastNotifications.error('Failed to download file', messages.tryAgain, downloadFile);
        }
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

    const copy = async () => {
        try {
            await makeCopy(bucket, path, file.name);
            ToastNotifications.notify(`${messages.copyOf} ${file.name} ${messages.wasCreated}`);
        } catch (error: any) {
            ToastNotifications.error('Error while copying file', `${messages.tryAgain}`, copy);
        };
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
        try {
            openModal(
                <DeleteFileModal
                    bucket={bucket}
                    file={file}
                    parrentFolder={parrentFolder}
                    path={path}
                />
            );
        } catch (error: any) { }
    };

    const viewFileVersions = async () => {
        try {

        } catch (error: any) { }
    };

    const share = async () => {
        try {
            const payload = await shareFile(bucket, [...path, file.name]);
            const link = `${window.location.origin}/api/v1/share?payload=${payload}`;
            openModal(
                <ShareFileModal link={link} />
            );
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
        downloadAction, moveToAction, makeCopyAction, renameAction, removeAction, shareAction,
    ];
    const warmInrecactiveActions = [
        downloadAction, moveToAction, makeCopyAction, renameAction, removeAction, shareAction,
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
        <div className="absolute w-48 right-8 text-xs font-medium bg-bucket-actionsBackground rounded-xl shadow-md z-10 text-bucket-actionsText select-none">{
            actions[bucketType].map(action =>
                <div
                    key={action.label}
                    className="w-full flex items-center gap-2 py-2 px-3 border-b-1 border-border-regular transition-all hover:bg-hover"
                    onClick={action.value}
                >
                    <span className="text-button-primary">
                        {action.icon}
                    </span>
                    {action.label}
                </div>
            )
        }
            <div
                className="w-full flex justify-between items-center gap-2 py-2 px-3 border-b-1 border-border-regular transition-all hover:bg-hover"
            >
                Your file is secure <span className="rounded-full w-2 h-2" style={{ background: '#2bb65e' }} />
            </div>
        </div>
    );
};
