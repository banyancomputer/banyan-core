import React, { useEffect, useState } from 'react';
import { useIntl } from 'react-intl';

import { Bucket, BucketFile } from '@/lib/interfaces/bucket';
import { getDateLabel } from '@/utils/date';
import { convertFileSize } from '@/utils/storage';

import { ActionsCell } from '@components/common/ActionsCell';
import { TrashFileActions } from '../TrashFileActions';
import { FileIcon } from '@/components/common/FileIcon';
import { SortCell } from '@/components/common/SortCell';

export const TrashTable: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { messages } = useIntl();
    const [selectedFiles, setSelectedFiles] = useState<BucketFile[]>([]);
    /** Created to prevent sotring logic affect initial buckets array */
    const [bucketCopy, setBucketCopy] = useState(bucket);

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
            setSelectedFiles(bucket.files);
    };
    const [sortState, setSortState] = useState<{ criteria: string; direction: 'ASC' | 'DESC' | '' }>({ criteria: '', direction: '' });

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

    return (
        <div className="max-h-[calc(100vh-367px)] w-fit overflow-x-auto border-2 border-gray-200 rounded-xl" >
            <div className="px-5 py-6 text-m font-semibold border-b-2 border-gray-200">
                {`${messages.files}`}
            </div>
            <div >
                <table className="table table-pin-rows w-full text-gray-600 rounded-xl  table-fixed ">
                    <thead className="border-b-table-cellBackground text-xxs font-normal ">
                        <tr className="border-b-table-cellBackground bg-table-headBackground font-normal">
                            <th className="flex items-center gap-3 px-6 py-4 text-left font-medium">
                                <input
                                    onChange={selectAll}
                                    type="checkbox"
                                    checked={selectedFiles.length === bucket.files.length}
                                    className="checkbox border-gray-600"
                                />
                                <SortCell
                                    criteria="name"
                                    onChange={sort}
                                    sortState={sortState}
                                    text={`${messages.name}`}
                                />
                            </th>
                            <th className="px-6 py-4 text-left font-medium w-36">
                                <SortCell
                                    criteria="deleted"
                                    onChange={sort}
                                    sortState={sortState}
                                    text={`${messages.dateDeleted}`}
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
                            <th className="px-6 py-4 text-left font-medium w-20"></th>
                        </tr>
                    </thead>
                    <tbody>
                        {
                            bucketCopy.files.map((file, index) =>
                                <tr key={index}>
                                    <td className="px-6 py-4 flex items-center gap-3">
                                        <input
                                            onChange={() => selectFile(file)}
                                            type="checkbox"
                                            checked={selectedFiles.includes(file)}
                                            className="checkbox border-gray-600"
                                        />
                                        <FileIcon fileName={file.name} className="p-2 bg-gray-200 rounded-full" />{file.name}
                                    </td>
                                    <td className="px-6 py-4">{getDateLabel(Date.now())}</td>
                                    <td className="px-6 py-4">{convertFileSize(file.metadata.size)}</td>
                                    <td className="px-6 py-4">
                                        <ActionsCell actions={<TrashFileActions bucket={bucket} file={file} />} />
                                    </td>
                                </tr>
                            )
                        }
                    </tbody>
                </table >
            </div>
        </div>
    );
};
