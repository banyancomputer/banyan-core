import React, { ReactElement, useMemo } from 'react';
import { useIntl } from 'react-intl';
import { FiDownload, FiEdit, FiTrash2 } from 'react-icons/fi';
import { AiOutlineLink } from 'react-icons/ai';
import { PiArrowsLeftRight, PiCopySimple } from 'react-icons/pi';
import { GoDotFill } from 'react-icons/go';
import { MdDone } from 'react-icons/md';
import { BiShareAlt } from 'react-icons/bi';

import { MoveToModal } from '../../common/Modal/MoveToModal';
import { RenameFileModal } from '../../common/Modal/RenameFileModal';
import { ShareFileModal } from '../Modal/ShareFileModal';
import { useTomb } from '@/contexts/tomb';
import { BrowserObject, Bucket } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';
import { ToastNotifications } from '@/utils/toastNotifications';
import { DeleteFileModal } from '@/components/common/Modal/DeleteFileModal';

export class Action {
    constructor(
        public label: string,
        public icon: ReactElement,
        public value: () => void,
        public tooltip?: string
    ) { }
}

export const FileActions: React.FC<{ bucket: Bucket; file: BrowserObject; parrentFolder: BrowserObject; path: string[] }> = ({ bucket, file, path, parrentFolder }) => {
    const { messages } = useIntl();
    const { download, makeCopy, shareFile } = useTomb();
    const { openModal, closeModal } = useModal();
    const bucketType = `${bucket.bucketType}_${bucket.storageClass}`;

    const downloadFile = async() => {
        try {
            await ToastNotifications.promise(`${messages.downloading}...`, `${messages.fileWasDownloaded}`, <MdDone size="20px" />,
                download(bucket, path, file.name)
            );
        } catch (error: any) { }
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

    const copy = async() => {
        try {
            await makeCopy(bucket, path, file.name);
            ToastNotifications.notify(`${messages.copyOf} ${file.name} ${messages.wasCreated}`, <AiOutlineLink size="20px" />);
        } catch (error: any) { }
    };

    const rename = async() => {
        openModal(
            <RenameFileModal
                bucket={bucket}
                file={file}
                path={path}
            />
        );
    };

    const remove = async() => {
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

    const viewFileVersions = async() => {
        try {

        } catch (error: any) { }
    };

    const share = async() => {
        try {
            const link = await shareFile(bucket, file);
            openModal(
                <ShareFileModal link={link} />
            );
        } catch (error: any) { }
    };

    const downloadAction = useMemo(() => new Action(`${messages.download}`, <FiDownload size="18px" />, downloadFile), []);
    const moveToAction = new Action(`${messages.moveTo}`, <PiArrowsLeftRight size="18px" />, moveTo);
    const makeCopyAction = new Action(`${messages.makeCopy}`, <PiCopySimple size="18px" />, copy);
    const vierFileVersionsAction = new Action(`${messages.viewFileVersions}`, <AiOutlineLink size="18px" />, viewFileVersions);
    const renameAction = new Action(`${messages.rename}`, <FiEdit size="18px" />, rename);
    const removeAction = new Action(`${messages.remove}`, <FiTrash2 size="18px" />, remove);
    const shareAction = new Action(`${messages.shareFile}`, <BiShareAlt size="18px" />, share);

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
                Your file is secure <GoDotFill fill="#2bb65e" />
        </div>
        </div>
    );
};
