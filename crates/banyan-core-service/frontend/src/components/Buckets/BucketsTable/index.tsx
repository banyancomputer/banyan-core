import React from 'react';

import { getDateLabel } from '@/utils/date';
import { Bucket as IBucket } from '@/lib/interfaces/bucket';
import { convertFileSize } from '@/utils/storage';
import { FileActionsCell } from './FileActionsCell';
import { BucketActionsCell } from './BucketActionsCell ';

export const BucketTable: React.FC<{ buckets: IBucket[] }> = ({ buckets }) =>
    <div className="max-h-[calc(100vh-367px)] overflow-x-auto border-2 border-c rounded-xl" >
        <table className="table table-pin-rows w-full text-gray-600 rounded-xl shadow-thead">
            <thead className="border-b-table-cellBackground text-xxs bg-table-headBackground">
                <tr className="border-b-table-cellBackground">
                    <th className="p-3 text-left">Bucket Name</th>
                    <th className="p-3 text-left">Name</th>
                    <th className="p-3 text-left">Storage Class</th>
                    <th className="p-3 text-left">Bucket Type</th>
                    <th className="p-3 text-left">Last Edited</th>
                    <th className="p-3 text-left">File Size</th>
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
    </div>;

