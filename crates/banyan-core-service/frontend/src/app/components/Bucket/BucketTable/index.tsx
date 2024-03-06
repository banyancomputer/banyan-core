import React, { useEffect, useMemo, useState } from 'react';
import { useParams } from 'react-router-dom';

import { ActionsCell } from '@components/common/ActionsCell';
import { BucketActions } from '@components/common/BucketActions';
import { SortCell } from '@components/common/SortCell';
import { FolderRow } from '@components/Bucket/BucketTable/FolderRow';
import { FileRow } from '@components/Bucket/BucketTable/FileRow';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { sortByType, sortFiles } from '@app/utils';
import { useFilesUpload } from '@app/contexts/filesUpload';
import { ToastNotifications } from '@utils/toastNotifications';
import { preventDefaultDragAction } from '@utils/dragHandlers';
import { useTomb } from '@app/contexts/tomb';
import { useAppSelector } from '@/app/store';

export const BucketTable: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const params = useParams();
    const bucketId = params.id;
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.bucketTable);
    const { uploadFiles, setFiles, files } = useFilesUpload();
    const { getSelectedBucketFiles, moveTo } = useTomb();
    /** Created to prevent sotring logic affect initial buckets array */
    const [bucketCopy, setBucketCopy] = useState(bucket);
    const [sortState, setSortState] = useState<{ criteria: string; direction: 'ASC' | 'DESC' | '' }>({ criteria: 'name', direction: 'DESC' });
    const folderLocation = useFolderLocation();
    const [areFilesDropped, setAreFilesDropped] = useState(false);
    const siblingFiles = useMemo(() => bucketCopy.files?.filter(file => file.type !== 'dir'), [bucketCopy.files]);

    const sort = (criteria: string) => {
        setSortState(prev => ({ criteria, direction: prev.direction === 'ASC' ? 'DESC' : 'ASC' }));
    };

    const handleDrop = async (event: React.DragEvent<HTMLDivElement>) => {
        preventDefaultDragAction(event);

        if (event?.dataTransfer.files.length) {
            setFiles(Array.from(event.dataTransfer.files).slice(0, 1).map(file => ({ file, status: 'pending' })));
            setAreFilesDropped(true);

            return;
        }

        const dragData = event.dataTransfer.getData('browserObject');
        if (dragData) {
            try {

                const droppedItem: { item: BrowserObject; path: string[] } = JSON.parse(dragData);

                if (!droppedItem.path.length) { return; }

                await moveTo(bucket, [...droppedItem.path, droppedItem.item.name], [], droppedItem.item.name);
                ToastNotifications.notify(messages.fileWasMoved);
                await getSelectedBucketFiles([]);
            } catch (error: any) {
                ToastNotifications.error(messages.moveToError, messages.tryAgain, () => handleDrop(event));
            };
        }
    };

    useEffect(() => {
        if (!files.length || !areFilesDropped) { return; }

        (async () => {
            try {
                await uploadFiles(bucket, folderLocation);
                setAreFilesDropped(false);
            } catch (error: any) {
                setAreFilesDropped(false);
                ToastNotifications.error(messages.uploadError, messages.tryAgain, () => { });
            }
        })();
    }, [files, areFilesDropped]);

    useEffect(() => {
        if (!bucket.files) { return; }
        setBucketCopy(bucket => ({
            ...bucket,
            files: [...bucket.files].sort((prev: BrowserObject, next: BrowserObject) => sortFiles(prev, next, sortState.criteria, sortState.direction !== 'ASC')).sort(sortByType),
        }));
    }, [sortState.criteria, sortState.direction, bucket]);

    useEffect(() => {
        setSortState(prev => ({ ...prev }));
    }, [bucketCopy]);

    useEffect(() => {
        setBucketCopy(bucket);
    }, [bucket]);

    useEffect(() => {
        setSortState({ criteria: 'name', direction: 'DESC' });
    }, [bucketId]);

    return (
        <div
            onDrop={handleDrop}
            onDragOver={preventDefaultDragAction}
            className={`w-fit h-full overflow-x-auto bg-secondaryBackground max-h-[calc(100vh-388px)]`}
            id="table"
        >
            <div >
                <table className="table table-pin-rows w-full text-text-600 rounded-xl table-fixed">
                    <thead className="border-b-border-regular text-xxs border-b-2 font-normal text-text-900">
                        <tr className="bg-secondaryBackground font-normal border-none">
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
                            <th className="px-6 py-4 text-left font-medium w-20">
                                <ActionsCell actions={<BucketActions bucket={bucket} />} />
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {
                            bucketCopy.files.map(file =>
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


