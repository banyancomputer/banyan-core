import React, { useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { ActionsCell } from '@components/common/ActionsCell';
import { FolderActions } from '../FolderActions';
import { FileCell } from '@components/common/FileCell';
import { FileRow } from '../FileRow';
import { DraggingPreview } from '../FileRow/DraggingPreview';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { getDateLabel } from '@/app/utils/date';
import { convertFileSize } from '@/app/utils/storage';
import { useTomb } from '@/app/contexts/tomb';
import { stringToHex } from '@utils/hex';
import { useFilesUpload } from '@app/contexts/filesUpload';
import { ToastNotifications } from '@utils/toastNotifications';
import { handleDrag, handleDragEnd, handleDragStart, preventDefaultDragAction } from '@utils/dragHandlers';
import { useAppSelector } from '@/app/store';

import { ChevronUp } from '@static/images/common';

export const FolderRow: React.FC<{
    folder: BrowserObject;
    bucket: Bucket;
    path: string[];
    nestingLevel?: number;
    parrentFolder?: BrowserObject;
}> = ({ folder, bucket, nestingLevel = 0, path = [], parrentFolder }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.files.bucketTable.folderRow);
    const folderRef = useRef<HTMLTableRowElement | null>(null);
    const navigate = useNavigate();
    const { getExpandedFolderFiles, getSelectedBucketFiles, moveTo, selectBucket } = useTomb();
    const { uploadFiles } = useFilesUpload();
    const [isFolderDraggingOver, setIsFolderDragingOver] = useState(false);
    const [isDragging, setIsDragging] = useState(false);
    const siblingFiles = useMemo(() => folder.files?.filter(file => file.type !== 'dir'), [folder.files]);

    const goToFolder = (event: React.MouseEvent<HTMLTableRowElement, MouseEvent>, bucket: Bucket) => {
        //@ts-ignore
        if (event.target.id === 'actionsCell') {
            return;
        };

        navigate(`/drive/${bucket.id}?${path.length ? `${path.map(element => stringToHex(element)).join('/')}/${stringToHex(folder.name)}` : stringToHex(folder.name)}`);
    };

    const expandFolder = async (event: any) => {
        event.stopPropagation();

        try {
            if (folder.files?.length) {
                folder.files = [];
                selectBucket({ ...bucket });
            } else {
                await getExpandedFolderFiles([...path, folder.name], folder, bucket);
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

                await moveTo(bucket, [...droppedItem.path, droppedItem.name], [...path, folder.name], droppedItem.name);
                await getSelectedBucketFiles(path);
                ToastNotifications.notify(messages.fileWasMoved);
            } catch (error: any) {
                ToastNotifications.error(messages.moveToError);
            }
        }
    };

    return (
        <tr
            className={`border-b-1 border-b-border-regular ${isFolderDraggingOver && 'bg-dragging border-draggingBorder'} transition-all `}
            onDragStart={event => handleDragStart(event, folder.name, setIsDragging, path)}
            onDrag={event => handleDrag(event, folder.name)}
            onDragOver={dragOverHandler}
            onDragLeave={dragLeaveHandler}
            onDragEnd={() => handleDragEnd(setIsDragging, getExpandedFolderFiles, getSelectedBucketFiles, path, parrentFolder, bucket)}
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
                            <td className="px-6 py-2">
                                {
                                    bucket.bucketType === 'backup' ?
                                        null
                                        :
                                        <ActionsCell
                                            actions={
                                                <FolderActions
                                                    bucket={bucket}
                                                    file={folder}
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
