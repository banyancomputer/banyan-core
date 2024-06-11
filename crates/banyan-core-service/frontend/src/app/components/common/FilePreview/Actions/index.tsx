import React, { useEffect, useRef, useState } from 'react';

import { Action } from '@components/Bucket/Files/BucketTable/FileActions';
import { DeleteFileModal } from '@components/common/Modal/DeleteFileModal';
import { RenameFileModal } from '@components/common/Modal/RenameFileModal';
import { MoveToModal } from '@components/common/Modal/MoveToModal';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { openModal } from '@store/modals/slice';
import { popupClickHandler } from '@/app/utils';

import { Dots, MoveTo, Rename, Trash } from '@/app/static/images/common';
import { useAppSelector } from '@/app/store';

export const FilePreviewActions: React.FC<{ bucket: Bucket; file: BrowserObject; parrentFolder: BrowserObject; path: string[] }> = ({ bucket, file, path, parrentFolder }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.filePreview.actions);
    const actionsRef = useRef<HTMLDivElement | null>(null);
    const [isVisible, setIsVisible] = useState(false);
    const bucketType = `${bucket.bucketType}_${bucket.storageClass}`;


    const toggleVisibility = () => {
        setIsVisible(prev => !prev);
    };

    const moveTo = () => {

        openModal({
            content: <MoveToModal
                file={file}
                bucket={bucket}
                path={path}
                parrentFolder={parrentFolder}
            />
        });
    };

    const rename = async () => {
        openModal({
            content: <RenameFileModal
                bucket={bucket}
                file={file}
                path={path}
            />
        });
    };

    const remove = async () => {
        try {
            openModal({
                content: <DeleteFileModal
                    bucket={bucket}
                    file={file}
                    parrentFolder={parrentFolder}
                    path={path}
                />,
                path: [bucket.name, ...path]
            });
        } catch (error: any) { }
    };

    const moveToAction = new Action(`${messages.moveTo}`, <MoveTo width="18px" height="18px" />, moveTo);
    const renameAction = new Action(`${messages.rename}`, <Rename width="18px" height="18px" />, rename);
    const removeAction = new Action(`${messages.remove}`, <Trash width="18px" height="18px" />, remove);

    const hotInrecactiveActions = [
        renameAction, moveToAction, removeAction,
    ];
    const warmInrecactiveActions = [
        renameAction, moveToAction, removeAction,
    ];
    const coldIntecactiveActions: Action[] = [];
    const hotBackupActions: Action[] = [];
    const warmBackupActions: Action[] = [];
    const coldBackupActions: Action[] = [];

    const actions: Record<string, Action[]> = {
        interactive_hot: hotInrecactiveActions,
        interactive_warm: warmInrecactiveActions,
        interactive_cold: coldIntecactiveActions,
        backup_hot: hotBackupActions,
        backup_warm: warmBackupActions,
        backup_cold: coldBackupActions,
    };


    useEffect(() => {
        if (!isVisible) return;

        const listener = popupClickHandler(actionsRef.current!, setIsVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [isVisible]);

    return (
        <div
            ref={actionsRef}
            className="relative text-white cursor-pointer z-40"
            onClick={toggleVisibility}
        >
            <Dots />
            {isVisible &&
                <div
                    className="absolute top-[130%] right-2 w-44 bg-bucket-actionsBackground text-bucket-actionsText rounded-lg"
                >
                    {
                        actions[bucketType].map(action =>
                            <div
                                key={action.label}
                                className="w-full flex items-center gap-2 py-2 px-3 border-border-regular transition-all font-medium text-xs hover:bg-hover last:border-t-1"
                                onClick={action.value}
                            >
                                <span className="text-button-primary">
                                    {action.icon}
                                </span>
                                {action.label}
                            </div>
                        )
                    }
                </div>
            }
        </div>
    )
}
