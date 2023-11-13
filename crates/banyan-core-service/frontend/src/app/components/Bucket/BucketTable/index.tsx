import React, { useEffect, useRef, useState } from 'react';
import { useIntl } from 'react-intl';
import { useParams } from 'react-router-dom';

import { ActionsCell } from '@components/common/ActionsCell';
import { BucketActions } from '@/app/components/common/BucketActions';
import { SortCell } from '@/app/components/common/SortCell';
import { FolderRow } from '@/app/components/common/FolderRow';
import { FileRow } from '@/app/components/common/FileRow';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { sortByType, sortFiles } from '@app/utils';

import { EmptyIcon } from '@static/images/common';

export const BucketTable: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const tableRef = useRef<HTMLDivElement | null>(null);
    const params = useParams();
    const bucketId = params.id;
    /** Created to prevent sotring logic affect initial buckets array */
    const [bucketCopy, setBucketCopy] = useState(bucket);
    const { messages } = useIntl();
    const [sortState, setSortState] = useState<{ criteria: string; direction: 'ASC' | 'DESC' | '' }>({ criteria: 'name', direction: 'DESC' });
    const [tableScroll, setTableScroll] = useState(0);
    const folderLocation = useFolderLocation();

    const sort = (criteria: string) => {
        setSortState(prev => ({ criteria, direction: prev.direction === 'ASC' ? 'DESC' : 'ASC' }));
    };

    useEffect(() => {
        if (!bucket.files) return;
        setBucketCopy(bucket => ({
            ...bucket,
            files: [...bucket.files].sort((prev: BrowserObject, next: BrowserObject) => sortFiles(prev, next, sortState.criteria, sortState.direction !== 'ASC')).sort(sortByType)
        }));
    }, [sortState, bucket]);

    useEffect(() => {
        setSortState(prev => ({ ...prev }));
    }, [bucketCopy]);

    useEffect(() => {
        setBucketCopy(bucket);
    }, [bucket]);

    useEffect(() => {
        setSortState({ criteria: 'name', direction: 'DESC' });
    }, [bucketId]);

    useEffect(() => {
        /** Weird typescript issue with scrollTop which exist, but not for typescript */
        // @ts-ignore
        tableRef.current?.addEventListener('scroll', event => setTableScroll(event.target.scrollTop));
    }, [tableRef]);

    return (
        <div
            ref={tableRef}
            className="max-h-[calc(100vh-210px)] w-fit overflow-x-auto bg-secondaryBackground border-2 border-border-regular rounded-xl shadow-common"
        >
            <div className="px-6 py-5 text-m font-semibold border-b-2 border-border-regular">
                {`${messages.files}`}
            </div>
            <div >
                <table className="table table-pin-rows w-full text-text-600 rounded-xl table-fixed">
                    <thead className="border-b-border-regular text-xxs border-b-2 font-normal text-text-900">
                        <tr className=" bg-secondaryBackground font-normal">
                            <th className="flex items-center gap-3 px-6 py-4 text-left font-medium">
                                <SortCell
                                    criteria="name"
                                    onChange={sort}
                                    sortState={sortState}
                                    text={`${messages.name}`}
                                />
                            </th>
                            <th className="px-6 py-4 text-left font-medium w-36">
                                <SortCell
                                    criteria="modified"
                                    onChange={sort}
                                    sortState={sortState}
                                    text={`${messages.lastEdited}`}
                                />
                            </th>
                            <th className="px-6 py-4 text-left font-medium w-36  ">
                                <SortCell
                                    criteria="size"
                                    onChange={sort}
                                    sortState={sortState}
                                    text={`${messages.fileSize}`}
                                />
                            </th>
                            <th className="px-6 py-4 text-left font-medium w-20">
                                <ActionsCell
                                    actions={<BucketActions bucket={bucket} />}
                                    offsetTop={tableScroll}
                                    tableRef={tableRef}
                                />
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
                                        tableRef={tableRef}
                                        tableScroll={tableScroll}
                                        path={folderLocation}
                                        key={file.name}
                                    />
                                    :
                                    <FileRow
                                        bucket={bucket}
                                        file={file}
                                        tableRef={tableRef}
                                        tableScroll={tableScroll}
                                        path={folderLocation}
                                        key={file.name}
                                    />
                            )
                        }
                    </tbody>
                </table>
            </div>
            {!bucketCopy.files.length ?
                <div className="h-full flex m-12 flex-col items-center justify-center saturate-0">
                    <EmptyIcon />
                    <p className="mt-4">{`${messages.driveIsEmpty}`}</p>
                </div>
                :
                null
            }
        </div>
    );
};

