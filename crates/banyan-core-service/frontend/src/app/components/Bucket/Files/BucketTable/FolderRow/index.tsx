import React, { useMemo, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { ActionsCell } from '@components/common/ActionsCell';
import { FolderActions } from '../FolderActions';
import { FileCell } from '@components/common/FileCell';
import { FileRow } from '../FileRow';
import { DraggingPreview } from '../FileRow/DraggingPreview';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { getDateLabel } from '@/app/utils/date';
import { convertFileSize } from '@/app/utils/storage';
import { stringToBase64 } from '@utils/base64';
import { useFilesUpload } from '@contexts/filesUpload';
import { ToastNotifications } from '@utils/toastNotifications';
import { handleDrag, handleDragEnd, handleDragStart, preventDefaultDragAction } from '@utils/dragHandlers';
import { useAppDispatch, useAppSelector } from '@/app/store';
import { selectBucket } from '@/app/store/tomb/slice';
import { getExpandedFolderFiles, getSelectedBucketFiles, moveTo } from '@/app/store/tomb/actions';
import { unwrapResult } from '@reduxjs/toolkit';

import { ChevronUp } from '@static/images/common';

export const FolderRow: React.FC<{
    folder: BrowserObject;
    bucket: Bucket;
    path: string[];
    nestingLevel?: number;
    parrentFolder?: BrowserObject;
}> = ({ folder, bucket, nestingLevel = 0, path = [], parrentFolder }) => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.files.bucketTable.folderRow);
    const folderRef = useRef<HTMLTableRowElement | null>(null);
    const navigate = useNavigate();
    const { uploadFiles } = useFilesUpload();
    const [isFolderDraggingOver, setIsFolderDragingOver] = useState(false);
    const [isDragging, setIsDragging] = useState(false);
    const siblingFiles = useMemo(() => folder.files?.filter(file => file.type !== 'dir'), [folder.files]);

    const goToFolder = (event: React.MouseEvent<HTMLTableRowElement, MouseEvent>, bucket: Bucket) => {
        //@ts-ignore
        if (event.target.id === 'actionsCell') {
            return;
        };

        navigate(`/drive/${bucket.id}?${path.length ? `${path.map(element => stringToBase64(element)).join('/')}/${stringToBase64(folder.name)}` : stringToBase64(folder.name)}`);
    };

    const expandFolder = async (event: any) => {
        event.stopPropagation();

        try {
            if (folder.files?.length) {
                folder.files = [];
                dispatch(selectBucket({ ...bucket }));
            } else {
                unwrapResult(await dispatch(getExpandedFolderFiles({path:[...path, folder.name], folder})));
            };
        } catch (error: any) {
            ToastNotifications.error(messages.failedToLoadFiles, messages.tryAgain, () => expandFolder(event));
        };
    };

    const dragOverHandler = (event: React.DragEvent<HTMLDivElement>) => {
        preventDefaultDragAction(event);
        setIsFolderDragingOver(true);
    };

    const dragLeaveHandler = (event: React.DragEvent<HTMLDivElement>) => {
        preventDefaultDragAction(event);
        setIsFolderDragingOver(false);
    };

    const handleDrop = async (event: React.DragEvent<HTMLDivElement>) => {
        preventDefaultDragAction(event);
        setIsFolderDragingOver(false);

        if (event?.dataTransfer.files.length) {
            try {
                await uploadFiles(event.dataTransfer.files, bucket, [...path, folder.name]);
            } catch (error: any) {
                ToastNotifications.error(`${messages.uploadError}`, `${messages.tryAgain}`, () => { });
            };

            return;
        };

        const dragData = event.dataTransfer.getData('browserObject');
        if (dragData) {
            try {
                const droppedItem: { name: string; path: string[] } = JSON.parse(dragData);
                if ([...path, folder.name].join('/') === droppedItem.path.join('/')) { return; }

                unwrapResult(await dispatch(moveTo({bucket, from: [...droppedItem.path, droppedItem.name], to: [...path, folder.name], name: droppedItem.name})));
                unwrapResult(await dispatch(getSelectedBucketFiles(path)));
                ToastNotifications.notify(messages.fileWasMoved);
            } catch (error: any) {
                ToastNotifications.error(messages.moveToError);
            };
        };
    };

    return (
        <tr
            className={`border-b-1 border-b-border-regular ${isFolderDraggingOver && 'bg-dragging border-draggingBorder'} transition-all `}
            onDragStart={event => handleDragStart(event, folder.name, setIsDragging, path)}
            onDrag={event => handleDrag(event, folder.name)}
            onDragOver={dragOverHandler}
            onDragLeave={dragLeaveHandler}
            onDragEnd={() => handleDragEnd(setIsDragging)}
            onDrop={handleDrop}
            ref={folderRef}
        >
            <td colSpan={4} className="p-0">
                <table className="w-full table table-fixed">
                    <thead>
                        <tr className=" bg-secondaryBackground font-normal border-none">
                            <th className="p-0" />
                            <th className="w-36 p-0" />
                            <th className="w-36 p-0" />
                            <th className="w-20 p-0" />
                        </tr>
                    </thead>
                    <tbody>
                        <tr
                            className={`cursor-pointer text-text-900 font-normal last:border-b-0 hover:bg-bucket-bucketHoverBackground ${folder?.files?.length && '!border-1 border-transparent border-b-border-regular'} ${isFolderDraggingOver && '!border-draggingBorder'}`}
                            onClick={event => goToFolder(event, bucket)}
                            draggable
                        >
                            <td
                                className="flex items-center gap-3 py-2"
                                style={{ paddingLeft: `${nestingLevel * 40}px` }}
                            >
                                <DraggingPreview name={folder.name} isDragging={isDragging} type="dir" />
                                <FileCell name={folder.name} type="dir" />
                                {!parrentFolder &&
                                    <span
                                        className={`${!folder.files?.length && 'rotate-180'} cursor-pointer p-2`}
                                        onClick={expandFolder}
                                    >
                                        <ChevronUp />
                                    </span>
                                }
                            </td>
                            <td className="px-6 py-2">{getDateLabel(+folder.metadata.modified)}</td>
                            <td className="px-6 py-2">{convertFileSize(folder.metadata.size)}</td>
                            <td className="px-6 py-0">
                                {
                                    bucket.bucketType === 'backup' ?
                                        null
                                        :
                                        <ActionsCell
                                            actions={
                                                <FolderActions
                                                    bucket={bucket}
                                                    folder={folder}
                                                    parrentFolder={parrentFolder!}
                                                    path={path}
                                                />
                                            }
                                        />
                                }
                            </td>
                        </tr>
                        {folder.files?.length ?
                            <>
                                {
                                    folder.files?.map((file, index) =>
                                        file.type === 'dir' ?
                                            <FolderRow
                                                bucket={bucket}
                                                folder={file}
                                                nestingLevel={nestingLevel + 1}
                                                parrentFolder={folder}
                                                path={[...path, folder.name]}
                                                key={index}
                                            />
                                            :
                                            <FileRow
                                                bucket={bucket}
                                                file={file}
                                                nestingLevel={nestingLevel + 1}
                                                parrentFolder={folder}
                                                siblingFiles={siblingFiles}
                                                path={[...path, folder.name]}
                                                key={index}
                                            />
                                    )
                                }
                            </>
                            :
                            null
                        }
                    </tbody>
                </table>
            </td>
        </tr>
    );
};
