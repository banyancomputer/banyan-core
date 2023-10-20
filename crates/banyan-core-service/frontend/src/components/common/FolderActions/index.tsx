import React, { useMemo } from 'react';
import { useIntl } from 'react-intl';
import { FiEdit, FiTrash2, FiUpload } from 'react-icons/fi';
import { PiArrowsLeftRight } from 'react-icons/pi';

import { MoveToModal } from '../Modal/MoveToModal';
import { RenameFileModal } from '../Modal/RenameFileModal';
import { Bucket, BucketFile } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';
import { DeleteFileModal } from '@/components/common/Modal/DeleteFileModal';
import { UploadFileModal } from '../Modal/UploadFileModal';
import { Action } from '../FileActions';

export const FolderActions: React.FC<{ bucket: Bucket; file: BucketFile }> = ({ bucket, file }) => {
    const { messages } = useIntl();
    const { openModal } = useModal();
    const bucketType = `${bucket.bucketType}_${bucket.storageClass}`;

    const uploadFile = () => {
        openModal(<UploadFileModal bucket={bucket} />);
    };

    const moveTo = () => {
        openModal(<MoveToModal file={file} bucket={bucket} />);
    };

    const rename = async () => {
        openModal(<RenameFileModal bucket={bucket} file={file} />);
    };

    const remove = async () => {
        openModal(<DeleteFileModal bucket={bucket} file={file} />);
    };

    const moveToAction = useMemo(() => new Action(`${messages.moveTo}`, <PiArrowsLeftRight size="18px" />, moveTo), []);
    const renameAction = useMemo(() => new Action(`${messages.rename}`, <FiEdit size="18px" />, rename), []);
    const removeAction = useMemo(() => new Action(`${messages.remove}`, <FiTrash2 size="18px" />, remove), []);
    const uploadFileAction = useMemo(() => new Action(`${messages.upload}`, <FiUpload size="18px" />, uploadFile), [])

    const hotInrecactiveActions = [
        uploadFileAction, moveToAction, renameAction, removeAction
    ];
    const warmInrecactiveActions = [
        uploadFileAction, moveToAction, renameAction, removeAction
    ];
    const coldIntecactiveActions = [
        moveToAction
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
        <div className="w-48 right-8 text-xs font-medium bg-mainBackground rounded-xl shadow-md z-10 select-none text-text-900">{
            actions[bucketType].map(action =>
                <div
                    key={action.label}
                    className="w-full flex items-center gap-2 py-2 px-3 border-b-1 border-border-regular transition-all hover:bg-hover"
                    onClick={action.value}
                    id='action'
                >
                    <span className='text-button-primary'>
                        {action.icon}
                    </span>
                    {action.label}
                </div>
            )
        }
        </div>
    );
};
