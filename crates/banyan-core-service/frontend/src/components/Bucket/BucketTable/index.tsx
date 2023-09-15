import React, { useEffect, useState } from 'react';
import { useSearchParams } from 'next/navigation';
import { useIntl } from 'react-intl';
import Image from 'next/image';
import Link from 'next/link';

import { ActionsCell } from '@components/common/ActionsCell';
import { Bucket } from '@/lib/interfaces/bucket';
import { getDateLabel } from '@/utils/date';
import { convertFileSize } from '@/utils/storage';
import { FileIcon } from '@/components/common/FileIcon';
import { SortCell } from '@/components/common/SortCell';
import { FileActions } from '@/components/common/FileActions';
import { BucketActions } from '@/components/common/BucketActions';
import { useFolderLocation } from '@/hooks/useFolderLocation';

import emptyIcon from '@sta tic/images/common/emptyIcon.png';

export const BucketTable: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const searchParams = useSearchParams();
    const bucketId = searchParams.get('id');
    /** Created to prevent sotring logic affect initial buckets array */
    const [bucketCopy, setBucketCopy] = useState(bucket);
    const { messages } = useIntl();
    const [sortState, setSortState] = useState<{ criteria: string; direction: 'ASC' | 'DESC' | '' }>({ criteria: '', direction: '' });
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

    return (
        <div className="max-h-[calc(100vh-210px)] w-fit overflow-x-auto border-2 border-gray-200 rounded-xl" >
            <div className="px-5 py-6 text-m font-semibold border-b-2 border-gray-200">
                {`${messages.files}`}
            </div>
            <div >
                <table className="table table-pin-rows w-full text-gray-600 rounded-xl  table-fixed ">
                    <thead className="border-b-table-cellBackground text-xxs font-normal ">
                        <tr className="border-b-table-cellBackground bg-table-headBackground font-normal">
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
                            <th className="px-6 py-4 text-left font-medium w-24">
                                <SortCell
                                    criteria="fileSize"
                                    onChange={sort}
                                    sortState={sortState}
                                    text={`${messages.fileSize}`}
                                />
                            </th>
                            <th className="px-6 py-4 text-left font-medium w-20">
                                <ActionsCell actions={<BucketActions bucket={bucket} />} />
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {
                            bucketCopy.files.map((file, index) =>
                                <tr key={index}>
                                    <td className="">
                                        {file.type === 'dir' ?
                                            <Link
                                                href={`/bucket/${bucket.id}?${folderLocation.join('/') ? `${folderLocation.join('/')}/` : ''}${file.name}`}
                                                className="px-6 py-4 flex items-center gap-3"
                                            >
                                                <FileIcon fileName={file.name} className="p-2 bg-gray-200 rounded-full" />{file.name}
                                            </Link>
                                            :
                                            <span className="px-6 py-4 flex items-center gap-3">
                                                <FileIcon fileName={file.name} className="p-2 bg-gray-200 rounded-full" />{file.name}
                                            </span>
                                        }
                                    </td>
                                    <td className="px-6 py-4">{getDateLabel(+file.metadata.modified)}</td>
                                    <td className="px-6 py-4">{convertFileSize(file.metadata.size)}</td>
                                    <td className="px-6 py-4">
                                        <ActionsCell actions={<FileActions bucket={bucket} file={file} />} />
                                    </td>
                                </tr>
                            )
                        }
                    </tbody>
                </table >
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
