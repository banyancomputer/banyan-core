import React, { ReactElement, useMemo } from 'react';
import { useIntl } from 'react-intl';
import { FiDownload, FiEdit, FiTrash2 } from 'react-icons/fi';
import { AiOutlineLink } from 'react-icons/ai';
import { PiArrowsLeftRight, PiCopySimple } from 'react-icons/pi';
import { GoDotFill } from 'react-icons/go';
import { MdDone } from 'react-icons/md';

import { MoveToModal } from '../../common/Modal/MoveToModal';
import { RenameFileModal } from '../../common/Modal/RenameFileModal';
import { useTomb } from '@/contexts/tomb';
import { Bucket, BucketFile } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';
import { ToastNotifications } from '@/utils/toastNotifications';
import { useFolderLocation } from '@/hooks/useFolderLocation';
import { DeleteFileModal } from '@/components/common/Modal/DeleteFileModal';

export class Action {
    constructor(
        public label: string,
        public icon: ReactElement,
        public value: () => void,
    ) { }
}

export const FileActions: React.FC<{ bucket: Bucket; file: BucketFile }> = ({ bucket, file }) => {
    const { messages } = useIntl();
    const { download, makeCopy } = useTomb();
    const { openModal } = useModal();
    const folredLoaction = useFolderLocation();
    const bucketType = `${bucket.bucketType}_${bucket.storageClass}`;

    const downloadFile = async () => {
        try {
            await ToastNotifications.promise(`${messages.downloading}...`, `${messages.fileWasDownloaded}`, <MdDone size="20px" />,
                download(bucket, folredLoaction, file.name)
            );
        } catch (error: any) { }
    };

    const copyLink = async () => {
        try {
            ToastNotifications.notify(`${messages.linkWasCopied}`, <AiOutlineLink size="20px" />);
        } catch (error: any) { }
    };

    const moveTo = () => {
        openModal(<MoveToModal file={file} bucket={bucket} />);
    };

    const copy = async () => {
        try {
            await makeCopy(bucket, folredLoaction, file.name);
            ToastNotifications.notify(`${messages.copyOf} ${file.name} ${messages.wasCreated}`, <AiOutlineLink size="20px" />);
        } catch (error: any) { }
    };

    const rename = async () => {
        openModal(<RenameFileModal bucket={bucket} file={file} />);
    };

    const remove = async () => {
        try {
            openModal(<DeleteFileModal bucket={bucket} file={file} />);
        } catch (error: any) { }
    };
    const viewFileVersions = async () => {
        try {

        } catch (error: any) { }
    };

    const downloadAction = useMemo(() => new Action(`${messages.download}`, <FiDownload size="18px" />, downloadFile), []);
    const copyLinkdAction = useMemo(() => new Action(`${messages.copyLink}`, <AiOutlineLink size="18px" />, copyLink), []);
    const moveToAction = useMemo(() => new Action(`${messages.moveTo}`, <PiArrowsLeftRight size="18px" />, moveTo), []);
    const makeCopyAction = useMemo(() => new Action(`${messages.makeCopy}`, <PiCopySimple size="18px" />, copy), []);
    const vierFileVersionsAction = useMemo(() => new Action(`${messages.viewFileVersions}`, <AiOutlineLink size="18px" />, viewFileVersions), []);
    const renameAction = useMemo(() => new Action(`${messages.rename}`, <FiEdit size="18px" />, rename), []);
    const removeAction = useMemo(() => new Action(`${messages.remove}`, <FiTrash2 size="18px" />, remove), []);

    const hotInrecactiveActions = [
        downloadAction, copyLinkdAction, moveToAction, makeCopyAction, renameAction, removeAction
    ];
    const warmInrecactiveActions = [
        downloadAction, moveToAction, makeCopyAction, renameAction, removeAction
    ];
    const coldIntecactiveActions = [
        downloadAction
    ];
    const hotBackupActions = [
        downloadAction, makeCopyAction
    ];
    const warmBackupActions = [
        downloadAction
    ];
    const coldBackupActions = [
        downloadAction
    ];

    const actions: Record<string, Action[]> = {
        interactive_hot: hotInrecactiveActions,
        interactive_warm: warmInrecactiveActions,
        interactive_cold: coldIntecactiveActions,
        backup_hot: hotBackupActions,
        backup_warm: warmBackupActions,
        backup_cold: coldBackupActions,
    }

    return (
        <div className="absolute w-48 right-8 text-xs font-medium bg-white rounded-xl shadow-md z-10 text-gray-900 select-none">{
            actions[bucketType].map(action =>
                <div
                    key={action.label}
                    className="w-full flex items-center gap-2 py-2 px-3 border-b-1 border-gray-200 transition-all hover:bg-slate-200"
                    onClick={action.value}
                >
                    {action.icon} {action.label}
                </div>
            )
        }
            <div
                className="w-full flex justify-between items-center gap-2 py-2 px-3 border-b-1 border-gray-200 transition-all hover:bg-slate-200"
            >
                Your file is secure <GoDotFill fill='#2bb65e' />
            </div>
        </div>
    );
};
