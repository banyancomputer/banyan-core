import React from 'react';
import { useIntl } from 'react-intl';

import { BucketActionsCell } from './BucketActionsCell ';
import { FileActionsCell } from './FileActionsCell';

import { getDateLabel } from '@/utils/date';
import { Bucket as IBucket } from '@/lib/interfaces/bucket';
import { convertFileSize } from '@/utils/storage';
import { FileIcon } from '../../common/FileIcon';

export const BucketsTable: React.FC<{ buckets: IBucket[] }> = ({ buckets }) => {
    const { messages } = useIntl();

    return (
        <div className="max-h-[calc(100vh-367px)] overflow-x-auto border-2 border-gray-200 rounded-xl" >
            <table className="table table-pin-rows w-full text-gray-600 rounded-xl shadow-thead">
                <thead className="border-b-table-cellBackground text-xxs font-normal">
                    <tr className="border-b-table-cellBackground bg-table-headBackground">
                        <th className="p-3 text-left font-medium">{`${messages.bucketName}`}</th>
                        <th className="p-3 text-left font-medium">{`${messages.name}`}</th>
                        <th className="p-3 text-left font-medium">{`${messages.storageClass}`}</th>
                        <th className="p-3 text-left font-medium">{`${messages.bucketType}`}</th>
                        <th className="p-3 text-left font-medium">{`${messages.lastEdited}`}</th>
                        <th className="p-3 text-left font-medium">{`${messages.fileSize}`}</th>
                        <th className="p-3 text-left font-medium"></th>
                    </tr>
                </thead>
                <tbody>
                    {buckets.map(bucket =>
                        <React.Fragment key={bucket.id}>
                            <tr className="bg-table-cellBackground">
                                <td className="px-3 py-4">{bucket.name}</td>
                                <td className="px-3 py-4"></td>
                                <td className="px-3 py-4"></td>
                                <td className="px-3 py-4">{bucket.bucket_type}</td>
                                <td className="px-3 py-4"></td>
                                <td className="px-3 py-4"></td>
                                <td className="px-3 py-4"><BucketActionsCell bucket={bucket} /></td>
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
                                        <td className="px-3 py-4"><FileActionsCell file={file} /></td>
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

