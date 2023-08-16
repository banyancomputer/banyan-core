import React, { useState } from 'react';
import { useIntl } from 'react-intl';

import { Bucket, BucketFile } from '@/lib/interfaces/bucket';
import { getDateLabel } from '@/utils/date';
import { convertFileSize } from '@/utils/storage';
import { FileActionsCell } from '../../Buckets/BucketsTable/FileActionsCell';
import { FileIcon } from '@/components/common/FileIcon';

export const TrashTable: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { messages } = useIntl();
    const [selectedFiles, setSelectedFiles] = useState<Array<BucketFile>>([]);

    const selectFile = (selectedFile: BucketFile) => {
        if (selectedFiles.includes(selectedFile)) {
            setSelectedFiles(files => files.filter(file => file !== selectedFile));
        } else {
            setSelectedFiles(files => [...files, selectedFile]);
        }
    };

    const selectAll = () => {
        selectedFiles.length === bucket.files.length ?
            setSelectedFiles([])
            :
            setSelectedFiles(bucket.files)
    };

    return (
        <div className="max-h-[calc(100vh-367px)] w-fit overflow-x-auto border-2 border-gray-200 rounded-xl" >
            <div className='px-5 py-6 text-m font-semibold border-b-2 border-gray-200'>
                Files
            </div>
            <div >
                <table className="table table-pin-rows w-full text-gray-600 rounded-xl shadow-thead table-fixed ">
                    <thead className="border-b-table-cellBackground text-xxs font-normal ">
                        <tr className="border-b-table-cellBackground bg-table-headBackground font-normal">
                            <th className="flex items-center gap-3 px-6 py-4 text-left font-medium">
                                <input
                                    onChange={selectAll}
                                    type="checkbox"
                                    checked={selectedFiles.length === bucket.files.length}
                                    className="checkbox border-gray-600"
                                />
                                {`${messages.fileName}`}
                            </th>
                            <th className="px-6 py-4 text-left font-medium w-36">{`${messages.dateDeleted}`}</th>
                            <th className="px-6 py-4 text-left font-medium w-24">{`${messages.fileSize}`}</th>
                            <th className="px-6 py-4 text-left font-medium w-20"></th>
                        </tr>
                    </thead>
                    <tbody>
                        {
                            bucket.files.map((file, index) =>
                                <tr key={index}>
                                    <td className="px-6 py-4 flex items-center gap-3">
                                        <input
                                            onChange={() => selectFile(file)}
                                            type="checkbox"
                                            checked={selectedFiles.includes(file)}
                                            className="checkbox border-gray-600"
                                        />
                                        <FileIcon fileName={file.name} className='p-2 bg-gray-200 rounded-full' />{file.name}
                                    </td>
                                    <td className="px-6 py-4">{getDateLabel(Date.now())}</td>
                                    <td className="px-6 py-4">{convertFileSize(file.metadata.size)}</td>
                                    <td className="px-6 py-4"><FileActionsCell file={file} /></td>
                                </tr>
                            )
                        }
                    </tbody>
                </table >
            </div>
        </div>
    )
}
