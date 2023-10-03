import React, { useEffect, useRef, useState } from 'react';
import { useSearchParams } from 'next/navigation';
import { useIntl } from 'react-intl';
import Image from 'next/image';
import { useRouter } from 'next/router';

import { ActionsCell } from '@components/common/ActionsCell';
import { FolderActions } from '@/components/common/FolderActions';
import { BucketActions } from '@/components/common/BucketActions';
import { SortCell } from '@/components/common/SortCell';
import { FileCell } from '@/components/common/FileCell';
import { FileActions } from '@/components/common/FileActions';

import { getDateLabel } from '@/utils/date';
import { Bucket, BucketFile } from '@/lib/interfaces/bucket';
import { useFilePreview } from '@/contexts/filesPreview';
import { convertFileSize } from '@/utils/storage';
import { useFolderLocation } from '@/hooks/useFolderLocation';

import emptyIcon from '@static/images/common/emptyIcon.png';

export const BucketTable: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const tableRef = useRef<HTMLDivElement | null>(null);
    const searchParams = useSearchParams();
    const bucketId = searchParams.get('id');
    /** Created to prevent sotring logic affect initial buckets array */
    const [bucketCopy, setBucketCopy] = useState(bucket);
    const { messages } = useIntl();
    const [sortState, setSortState] = useState<{ criteria: string; direction: 'ASC' | 'DESC' | '' }>({ criteria: '', direction: '' });
    const folderLocation = useFolderLocation();
    const { push } = useRouter();
    const { openFile } = useFilePreview();
    const [tableScroll, setTableScroll] = useState(0);

    const sort = (criteria: string) => {
        setSortState(prev => ({ criteria, direction: prev.direction === 'ASC' ? 'DESC' : 'ASC' }));
    };

    const goTofolder = (bucket: Bucket, folder: BucketFile) => {
        push(`/bucket/${bucket.id}?${folderLocation.length ? `${folderLocation.join('/')}/${folder.name}` : folder.name}`);
    };

    const previewFile = async (bucket: Bucket, file: BucketFile) => {
        openFile(bucket, file.name, folderLocation);
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
        //@ts-ignore
        tableRef.current?.addEventListener("scroll", event => setTableScroll(event.target.scrollTop));
    }, [tableRef]);

    return (
        <div
            ref={tableRef}
            className="max-h-[calc(100vh-210px)] w-fit overflow-x-auto border-2 border-gray-200 rounded-xl "
        >
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
                            bucketCopy.files.map((file, index) => {
                                return (<tr key={index}>
                                    <td
                                        onClick={() => file.type === 'dir' ? goTofolder(bucket, file) : previewFile(bucket, file)}
                                        className='px-6 py-4'
                                    >
                                        <FileCell name={file.name} />
                                    </td>
                                    <td className="px-6 py-4">{getDateLabel(+file.metadata.modified)}</td>
                                    <td className="px-6 py-4">{convertFileSize(file.metadata.size)}</td>
                                    <td className="px-6 py-4">
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
                                </tr>);
                            })
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
