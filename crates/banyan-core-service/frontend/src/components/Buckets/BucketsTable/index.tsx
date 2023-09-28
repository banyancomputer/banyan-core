import React, { useEffect, useRef, useState } from 'react';
import { useIntl } from 'react-intl';
import { useRouter } from 'next/router';

import { ActionsCell } from '@components/common/ActionsCell';
import { FileCell } from '@/components/common/FileCell';
import { BucketActions } from '@components/common/BucketActions';
import { SortCell } from '@/components/common/SortCell';
import { FileActions } from '@/components/common/FileActions';
import { FolderActions } from '@/components/common/FolderActions';

import { getDateLabel } from '@/utils/date';
import { Bucket, BucketFile, Bucket as IBucket } from '@/lib/interfaces/bucket';
import { convertFileSize } from '@/utils/storage';
import { useFilePreview } from '@/contexts/filesPreview';

export const BucketsTable: React.FC<{ buckets: IBucket[] }> = ({ buckets }) => {
    const tableRef = useRef<HTMLDivElement | null>(null);
    const { messages } = useIntl();
    const { push } = useRouter();
    /** Created to prevent sotring logic affect initial buckets array */
    const [bucketsCopy, setBucketsCopy] = useState(buckets);
    const { openFile } = useFilePreview();
    const [sortState, setSortState] = useState<{ criteria: string; direction: 'ASC' | 'DESC' | '' }>({ criteria: '', direction: '' });
    const [tableScroll, setTableScroll] = useState(0);

    const sort = (criteria: string) => {
        setSortState(prev => ({ criteria, direction: prev.direction === 'ASC' ? 'DESC' : 'ASC' }));
    };

    const goToBucket = (bucket: string) => {
        push(`/bucket/${bucket}`);
    };

    const goTofolder = (bucket: Bucket, folder: BucketFile) => {
        push(`/bucket/${bucket.id}?${folder.name}`);
    };

    const previewFile = async (bucket: Bucket, file: BucketFile) => {
        openFile(bucket, file.name, []);
    };

    useEffect(() => {
        if (sortState.criteria === 'name') {
            setBucketsCopy(prev => prev.map(bucket => {
                const files = [...bucket.files];
                files.sort((a, b) => sortState.direction !== 'ASC' ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name));

                return { ...bucket, files };
            }));

            return;
        }

        setBucketsCopy(prev => prev.map(bucket => {
            const files = [...bucket.files];
            files.sort((a, b) => sortState.direction !== 'ASC' ? Number(a.metadata[sortState.criteria]) - Number(b.metadata[sortState.criteria]) : Number(b.metadata[sortState.criteria]) - Number(a.metadata[sortState.criteria]));

            return { ...bucket, files };
        }));
    }, [sortState, buckets]);

    useEffect(() => {
        setBucketsCopy(buckets);
    }, [buckets]);


    useEffect(() => {
        /** Weird typescript issue with scrollTop which exist, but not for typescript */
        //@ts-ignore
        tableRef.current?.addEventListener("scroll", event => setTableScroll(event.target.scrollTop));
    }, [tableRef]);

    return (
        <div
            ref={tableRef}
            className="max-h-[calc(100vh-210px)] w-full overflow-x-auto border-2 border-gray-200 rounded-xl"
        >
            <table className="table table-pin-rows w-full text-gray-600 rounded-xl table-fixed">
                <thead className="border-b-table-cellBackground text-xxs font-normal">
                    <tr className="border-b-table-cellBackground bg-table-headBackground">
                        <th className="p-3 text-left font-medium">{`${messages.bucketName}`}</th>
                        <th className="p-3 text-left font-medium">
                            <SortCell
                                criteria="name"
                                onChange={sort}
                                sortState={sortState}
                                text={`${messages.name}`}
                            />
                        </th>
                        <th className="p-3 text-left font-medium w-36">{`${messages.storageClass}`}</th>
                        <th className="p-3 text-left font-medium w-36">{`${messages.bucketType}`}</th>
                        <th className="p-3 text-left font-medium w-36">
                            <SortCell
                                criteria="lastEdited"
                                onChange={sort}
                                sortState={sortState}
                                text={`${messages.lastEdited}`}
                            />
                        </th>
                        <th className="p-3 text-left font-medium w-24">
                            <SortCell
                                criteria="fileSize"
                                onChange={sort}
                                sortState={sortState}
                                text={`${messages.fileSize}`}
                            />
                        </th>
                        <th className="p-3 text-left font-medium w-20"></th>
                    </tr>
                </thead>
                <tbody>
                    {bucketsCopy.map(bucket =>
                        <React.Fragment key={bucket.id}>
                            <tr className="bg-table-cellBackground">
                                <td
                                    className="px-3 py-4  cursor-pointer"
                                    onClick={() => goToBucket(bucket.id)}
                                >{bucket.name}</td>
                                <td className="px-3 py-4"></td>
                                <td className="px-3 py-4">{bucket.storageClass}</td>
                                <td className="px-3 py-4">{bucket.bucketType}</td>
                                <td className="px-3 py-4"></td>
                                <td className="px-3 py-4"></td>
                                <td
                                    className="px-3 py-4"
                                >
                                    <ActionsCell
                                        actions={<BucketActions bucket={bucket} />}
                                        offsetTop={tableScroll}
                                        tableRef={tableRef}
                                    />
                                </td>
                            </tr>
                            {
                                bucket.files.map((file, index) =>
                                    <tr key={index}>
                                        <td className="px-3 py-4"></td>
                                        <td
                                            className="px-3 py-4"
                                            onClick={() => file.type === 'dir' ? goTofolder(bucket, file) : previewFile(bucket, file)}
                                        >
                                            <FileCell name={file.name} />
                                        </td>
                                        <td className="px-3 py-4"></td>
                                        <td className="px-3 py-4"></td>
                                        <td className="px-3 py-4">{getDateLabel(Number(file.metadata.modified))}</td>
                                        <td className="px-3 py-4">{file.type === 'dir' ? '' : convertFileSize(file.metadata.size)}</td>
                                        <td
                                            className="px-3 py-4"
                                        >
                                            {
                                                file.type === 'dir' && bucket.bucketType === 'backup' ?
                                                    null
                                                    :
                                                    <ActionsCell
                                                        actions={file.type === 'dir' ? <FolderActions bucket={bucket} file={file} /> : <FileActions bucket={bucket} file={file} />}
                                                        offsetTop={tableScroll}
                                                        tableRef={tableRef}
                                                    />
                                            }
                                        </td>
                                    </tr>
                                )
                            }
                        </React.Fragment>
                    )}
                </tbody>
            </table >
        </div >
    );
};

