import React, { useEffect, useRef, useState } from 'react';
import { useRouter } from 'next/router';
import { useIntl } from 'react-intl';
import Image from 'next/image';

import { ActionsCell } from '@components/common/ActionsCell';
import { BucketActions } from '@/components/common/BucketActions';
import { SortCell } from '@/components/common/SortCell';
import { FolderRow } from '@/components/common/FolderRow';
import { FileRow } from '@/components/common/FileRow';

import { Bucket } from '@/lib/interfaces/bucket';
import { useFolderLocation } from '@/hooks/useFolderLocation';

import emptyIcon from '@static/images/common/emptyIcon.png';

export const BucketTable: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const tableRef = useRef<HTMLDivElement | null>(null);
    const router = useRouter();
    const bucketId = router.query.id;
    /** Created to prevent sotring logic affect initial buckets array */
    const [bucketCopy, setBucketCopy] = useState(bucket);
    const { messages } = useIntl();
    const [sortState, setSortState] = useState<{ criteria: string; direction: 'ASC' | 'DESC' | '' }>({ criteria: '', direction: '' });
    const [tableScroll, setTableScroll] = useState(0);
    const folderLocation = useFolderLocation();

    const sort = (criteria: string) => {
        setSortState(prev => ({ criteria, direction: prev.direction === 'ASC' ? 'DESC' : 'ASC' }));
    };

    useEffect(() => {
        if (sortState.criteria === 'name') {
            setBucketCopy(bucket => {
                const files = [...bucket.files];
                files.sort((a, b) => sortState.direction !== 'ASC' ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name));

                return { ...bucket, files };
            });
        } else {
            setBucketCopy(bucket => {
                const files = [...bucket.files];
                files.sort((a, b) => sortState.direction !== 'ASC' ? Number(a.metadata[sortState.criteria]) - Number(b.metadata[sortState.criteria]) : Number(b.metadata[sortState.criteria]) - Number(a.metadata[sortState.criteria]));

                return { ...bucket, files };
            });
        }
    }, [sortState, bucket]);

    useEffect(() => {
        setBucketCopy(bucket);
    }, [bucket]);

    useEffect(() => {
        setSortState({ criteria: '', direction: '' });
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
                    <thead className="border-b-border-regular text-xxs font-normal text-text-900">
                        <tr className="border-b-border-regular bg-secondaryBackground font-normal">
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
                            <th className="px-6 py-4 text-left font-medium w-36">
                                <SortCell
                                    criteria="fileSize"
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
                            bucketCopy.files.map((file, index) =>
                                file.type === 'dir' ?
                                    <FolderRow
                                        bucket={bucket}
                                        folder={file}
                                        tableRef={tableRef}
                                        tableScroll={tableScroll}
                                        path={folderLocation}
                                        key={index}
                                    />
                                    :
                                    <FileRow
                                        bucket={bucket}
                                        file={file}
                                        tableRef={tableRef}
                                        tableScroll={tableScroll}
                                        path={folderLocation}
                                        key={index}
                                    />
                            )
                        }
                    </tbody>
                </table>
            </div>
            {!bucketCopy.files.length ?
                <div className="h-full flex m-12 flex-col items-center justify-center saturate-0">
                    <Image src={emptyIcon} alt="emptyIcon" />
                    <p className="mt-4">{`${messages.bucketIsEmpty}`}</p>
                </div>
                :
                null
            }
        </div>
    );
};

