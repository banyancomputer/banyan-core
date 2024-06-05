import React, { useEffect, useMemo, useState } from 'react';
import { useParams } from 'react-router-dom';
import { unwrapResult } from '@reduxjs/toolkit';

import { ActionsCell } from '@components/common/ActionsCell';
import { BucketActions } from '@components/common/BucketActions';
import { SortCell } from '@components/common/SortCell';
import { FolderRow } from '@components/Bucket/Files/BucketTable/FolderRow';
import { FileRow } from '@components/Bucket/Files/BucketTable/FileRow';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { sortByType, sortFiles } from '@app/utils';
import { ToastNotifications } from '@utils/toastNotifications';
import { preventDefaultDragAction } from '@utils/dragHandlers';
import { useAppDispatch, useAppSelector } from '@store/index';
import { getSelectedBucketFiles, moveTo } from '@store/tomb/actions';
import { setBucketFiles } from '@store/tomb/slice';
import { uploadFiles } from '@store/filesUpload/actions';

export const BucketTable: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const params = useParams();
    const dispatch = useAppDispatch();
    const bucketId = params.id;
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.files.bucketTable);
    const [sortState, setSortState] = useState<{ criteria: string; direction: 'ASC' | 'DESC' | '' }>({ criteria: 'name', direction: 'DESC' });
    const folderLocation = useFolderLocation();
    const siblingFiles = useMemo(() => bucket.files?.filter(file => file.type !== 'dir'), [bucket.files]);

    const sort = (criteria: string) => {
        setSortState(prev => ({ criteria, direction: prev.direction === 'ASC' ? 'DESC' : 'ASC' }));
    };

    const handleDrop = async (event: React.DragEvent<HTMLDivElement>) => {
        preventDefaultDragAction(event);

        if (event?.dataTransfer.files.length) {
            try {
                unwrapResult(await dispatch(uploadFiles({ fileList: event.dataTransfer.files, bucket, path: folderLocation, folderLocation })));
            } catch (error: any) {
                ToastNotifications.error(messages.uploadError);
            };

            return;
        };

        const dragData = event.dataTransfer.getData('browserObject');
        if (dragData) {
            try {
                const droppedItem: { name: string; path: string[] } = JSON.parse(dragData);
                if (!droppedItem.path.length) { return; }

                unwrapResult(await dispatch(moveTo({ bucket, from: [...droppedItem.path, droppedItem.name], to: [], name: droppedItem.name })));
                ToastNotifications.notify(messages.fileWasMoved);
                unwrapResult(await dispatch(getSelectedBucketFiles([])));
            } catch (error: any) {
                ToastNotifications.error(messages.moveToError, messages.tryAgain, () => handleDrop(event));
            };
        };
    };

    useEffect(() => {
        if (!bucket.files) { return; }
        dispatch(setBucketFiles(
            [...bucket.files].sort((prev: BrowserObject, next: BrowserObject) => sortFiles(prev, next, sortState.criteria, sortState.direction !== 'ASC')).sort(sortByType),
        ));
    }, [sortState.criteria, sortState.direction]);

    useEffect(() => {
        setSortState({ criteria: 'name', direction: 'DESC' });
    }, [bucketId]);

    return (
        <div
            onDrop={handleDrop}
            onDragOver={preventDefaultDragAction}
            className={`w-fit h-full overflow-x-auto bg-mainBackground max-h-[calc(100vh-388px)]`}
            id="table"
        >
            <div >
                <table className="table table-pin-rows w-full text-text-600 rounded-xl table-fixed">
                    <thead className="border-b-border-regular text-xxs border-b-2 font-normal text-text-900">
                        <tr className="bg-mainBackground font-normal border-none">
                            <th className="flex items-center gap-3 pl-0 py-4 text-left font-medium">
                                <SortCell
                                    criteria="name"
                                    onChange={sort}
                                    sortState={sortState}
                                    text={messages.name}
                                />
                            </th>
                            <th className="px-6 py-4 text-left font-medium w-36">
                                <SortCell
                                    criteria="modified"
                                    onChange={sort}
                                    sortState={sortState}
                                    text={messages.lastModified}
                                />
                            </th>
                            <th className="px-6 py-4 text-left font-medium w-36  ">
                                <SortCell
                                    criteria="size"
                                    onChange={sort}
                                    sortState={sortState}
                                    text={messages.fileSize}
                                />
                            </th>
                            <th className="px-6 py-0 text-left font-medium w-20">
                                <ActionsCell actions={<BucketActions bucket={bucket} />} />
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {
                            bucket.files.map(file =>
                                file.type === 'dir' ?
                                    <FolderRow
                                        bucket={bucket}
                                        folder={file}
                                        path={folderLocation}
                                        key={file.name}
                                    />
                                    :
                                    <FileRow
                                        bucket={bucket}
                                        file={file}
                                        path={folderLocation}
                                        siblingFiles={siblingFiles}
                                        key={file.name}
                                    />
                            )
                        }
                    </tbody>
                </table>
            </div>
        </div>
    );
};


