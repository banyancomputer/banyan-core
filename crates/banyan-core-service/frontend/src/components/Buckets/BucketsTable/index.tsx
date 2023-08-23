import React, { useEffect, useState } from 'react';
import { useIntl } from 'react-intl';


import { ActionsCell } from '../ActionsCell';

import { getDateLabel } from '@/utils/date';
import { Bucket as IBucket } from '@/lib/interfaces/bucket';
import { convertFileSize } from '@/utils/storage';
import { FileIcon } from '../../common/FileIcon';
import { SortCell } from '@/components/common/SortCell';
import { FileActions } from '@/components/Buckets/FileActions';
import { BucketActions } from '../BucketActions';

export const BucketsTable: React.FC<{ buckets: IBucket[] }> = ({ buckets }) => {

    const { messages } = useIntl();
    /** Created to prevent sotring logic affect initial buckets array */
    const [bucketsCopy, setBucketsCopy] = useState(buckets);
    const [sortState, setSortState] = useState<{ criteria: string, direction: "ASC" | "DESC" | '' }>({ criteria: '', direction: '' });

    const sort = (criteria: string) => {
        setSortState(prev => ({ criteria, direction: prev.direction === "ASC" ? "DESC" : "ASC" }));
    };

    useEffect(() => {
        if (sortState.criteria === 'name') {
            setBucketsCopy(prev => prev.map(bucket => {
                const files = [...bucket.files];
                files.sort((a, b) => sortState.direction !== 'ASC' ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name));

                return { ...bucket, files }
            }));

            return;
        }

        setBucketsCopy(prev => prev.map(bucket => {
            const files = [...bucket.files];
            files.sort((a, b) => sortState.direction !== 'ASC' ? Number(a.metadata[sortState.criteria]) - Number(b.metadata[sortState.criteria]) : Number(b.metadata[sortState.criteria]) - Number(a.metadata[sortState.criteria]));

            return { ...bucket, files }
        }));
    }, [sortState, buckets]);

    useEffect(() => {
        setBucketsCopy(buckets);
    }, [buckets]);

    return (
        <div className="max-h-[calc(100vh-210px)] overflow-x-auto border-2 border-gray-200 rounded-xl" >
            <table className="table table-pin-rows w-full text-gray-600 rounded-xl ">
                <thead className="border-b-table-cellBackground text-xxs font-normal">
                    <tr className="border-b-table-cellBackground bg-table-headBackground">
                        <th className="p-3 text-left font-medium">{`${messages.bucketName}`}</th>
                        <th className="p-3 text-left font-medium">
                            <SortCell
                                criteria='name'
                                onChange={sort}
                                sortState={sortState}
                                text={`${messages.name}`}
                            />
                        </th>
                        <th className="p-3 text-left font-medium">{`${messages.storageClass}`}</th>
                        <th className="p-3 text-left font-medium">{`${messages.bucketType}`}</th>
                        <th className="p-3 text-left font-medium">
                            <SortCell
                                criteria='lastEdited'
                                onChange={sort}
                                sortState={sortState}
                                text={`${messages.lastEdited}`}
                            />
                        </th>
                        <th className="p-3 text-left font-medium">
                            <SortCell
                                criteria='fileSize'
                                onChange={sort}
                                sortState={sortState}
                                text={`${messages.fileSize}`}
                            />
                        </th>
                        <th className="p-3 text-left font-medium"></th>
                    </tr>
                </thead>
                <tbody>
                    {bucketsCopy.map(bucket =>
                        <React.Fragment key={bucket.id}>
                            <tr className="bg-table-cellBackground">
                                <td className="px-3 py-4">{bucket.name}</td>
                                <td className="px-3 py-4"></td>
                                <td className="px-3 py-4"></td>
                                <td className="px-3 py-4">{bucket.bucket_type}</td>
                                <td className="px-3 py-4"></td>
                                <td className="px-3 py-4"></td>
                                <td className="px-3 py-4">
                                    <ActionsCell actions={<BucketActions bucket={bucket} />} />
                                </td>
                            </tr>
                            {
                                bucket.files.map((file, index) =>
                                    <tr key={index}>
                                        <td className="px-3 py-4"></td>
                                        <td className="px-3 py-4 flex items-center gap-3 "><FileIcon fileName={file.name} /> {file.name} </td>
                                        <td className="px-3 py-4"></td>
                                        <td className="px-3 py-4"></td>
                                        <td className="px-3 py-4">{getDateLabel(Number(file.metadata.modified))}</td>
                                        <td className="px-3 py-4">{convertFileSize(file.metadata.size)}</td>
                                        <td className="px-3 py-4"><ActionsCell actions={<FileActions bucket={bucket} file={file} />} /></td>
                                    </tr>
                                )
                            }
                        </React.Fragment>
                    )}
                </tbody>
            </table >
        </div>
    )
}

