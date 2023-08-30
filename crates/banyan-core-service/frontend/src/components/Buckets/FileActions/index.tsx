import React, { ReactElement, useCallback } from 'react';
import { useIntl } from 'react-intl';
import { FiDownload, FiEdit, FiTrash2 } from 'react-icons/fi';
import { AiOutlineLink } from 'react-icons/ai';
import { PiArrowsLeftRight, PiCopySimple } from 'react-icons/pi';
import { MdDone } from 'react-icons/md';

import { MoveToModal } from '../../common/Modal/MoveToModal';
import { RenameFileModal } from '../../common/Modal/RenameFileModal';
import { useTomb } from '@/contexts/tomb';
import { Bucket, BucketFile } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';
import { ToastNotifications } from '@/utils/toastNotifications';

export class FileAction {
    constructor(
        public label: string,
        public icon: ReactElement,
        public value: () => void,
    ) { }
}

export const FileActions: React.FC<{ bucket: Bucket; file: BucketFile }> = ({ bucket, file }) => {
    const { messages } = useIntl();
    const { } = useTomb();
    const { openModal } = useModal();

    const download = async () => {
        try {
            await ToastNotifications.promise(`${messages.downloading}...`, `${messages.fileWasDownloaded}`, <MdDone size="20px" />,
                () => new Promise(resolve => setTimeout(resolve, 3000))
            );
        } catch (error: any) { }
    };

    const copyLink = async () => {
        try {
            ToastNotifications.notify(`${messages.linkWasCopied}`, <AiOutlineLink size="20px" />);
        } catch (error: any) { }
    };

    const moveTo = () => {
        openModal(<MoveToModal file={file} />);
    };
    const makeCopy = async () => {
        try {
            ToastNotifications.notify(`${messages.copyOf} ${file.name} ${messages.wasCreated}`, <AiOutlineLink size="20px" />);
        } catch (error: any) { }
    };
    const viewFileVersions = async () => {
        try {

        } catch (error: any) { }
    };
    const rename = async () => {
        openModal(<RenameFileModal bucket={bucket} file={file} />);
    };

    const remove = async () => {
        try {

        } catch (error: any) { }
    };

    const acrions = [
        new FileAction(`${messages.download}`, <FiDownload size="18px" />, download),
        new FileAction(`${messages.copyLink}`, <AiOutlineLink size="18px" />, copyLink),
        new FileAction(`${messages.moveTo}`, <PiArrowsLeftRight size="18px" />, moveTo),
        new FileAction(`${messages.makeCopy}`, <PiCopySimple size="18px" />, makeCopy),
        new FileAction(`${messages.viewFileVersions}`, <AiOutlineLink size="18px" />, viewFileVersions),
        new FileAction(`${messages.rename}`, <FiEdit size="18px" />, rename),
        new FileAction(`${messages.remove}`, <FiTrash2 size="18px" />, remove),
    ];

    return (
        <div className="relative w-48 text-xs font-medium bg-white rounded-xl shadow-md z-10 text-gray-900">{
            acrions.map(action =>
                <div
                    key={action.label}
                    className="w-full flex items-center gap-2 py-2 px-3 border-b-1 border-gray-200 transition-all hover:bg-slate-200"
                    onClick={action.value}
                >
                    {action.icon} {action.label}
                </div>
            )
        }</div>
    );
};
