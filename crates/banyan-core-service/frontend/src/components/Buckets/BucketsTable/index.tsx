import React from 'react';
import { useIntl } from 'react-intl';

import { BucketActionsCell } from './BucketActionsCell ';
import { FileActionsCell } from './FileActionsCell';

import { getDateLabel } from '@/utils/date';
import { Bucket as IBucket } from '@/lib/interfaces/bucket';
import { convertFileSize } from '@/utils/storage';

export const BucketTable: React.FC<{ buckets: IBucket[] }> = ({ buckets }) => {
    const { messages } = useIntl();

    return (
        <div className="max-h-[calc(100vh-367px)] overflow-x-auto border-2 border-gray-200 rounded-xl" >
            <table className="table table-pin-rows w-full text-gray-600 rounded-xl shadow-thead">
                <thead className="border-b-table-cellBackground text-xxs font-normal bg-table-headBackground">
                    <tr className="border-b-table-cellBackground">
                        <th className="p-3 text-left">{`${messages.bucketName}`}</th>
                        <th className="p-3 text-left">{`${messages.name}`}</th>
                        <th className="p-3 text-left">{`${messages.storageClass}`}</th>
                        <th className="p-3 text-left">{`${messages.bucketType}`}</th>
                        <th className="p-3 text-left">{`${messages.lastEdited}`}</th>
                        <th className="p-3 text-left">{`${messages.fileSize}`}</th>
                        <th className="p-3 text-left"></th>
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
                                        <td className="px-3 py-4">{file.name}</td>
                                        <td className="px-3 py-4"></td>
                                        <td className="px-3 py-4"></td>
                                        <td className="px-3 py-4">{getDateLabel(file.metadata.modified)}</td>
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

