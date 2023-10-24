import React, { useState } from 'react';
import { useRouter } from 'next/router';

import { ActionsCell } from '../ActionsCell';
import { FolderActions } from '../FolderActions';
import { FileCell } from '../FileCell';
import { FileRow } from '../FileRow';

import { BrowserObject, Bucket } from '@/lib/interfaces/bucket';
import { getDateLabel } from '@/utils/date';
import { convertFileSize } from '@/utils/storage';
import { useTomb } from '@/contexts/tomb';

import { ChevronUp } from '@static/images/common';

export const FolderRow: React.FC<{
    folder: BrowserObject,
    bucket: Bucket,
    tableScroll: number,
    tableRef: React.MutableRefObject<HTMLDivElement | null>,
    nestingLevel: number,
    path: string[]
    parrentFolder?: BrowserObject,
}> = ({ folder, bucket, tableRef, tableScroll, nestingLevel = 0.25, path = [], parrentFolder }) => {
    const [isVisible, setIsVisible] = useState(false);
    const { push } = useRouter();

    const { getExpandedFolderFiles, selectBucket } = useTomb();

    const goToFolder = (event: React.MouseEvent<HTMLTableRowElement, MouseEvent>, bucket: Bucket) => {
        //@ts-ignore
        if (event.target.id === 'actionsCell') return;

        push(`/bucket/${bucket.id}?${path.length ? `${path.join('/')}/${folder.name}` : folder.name}`);
    };

    const expandFolder = async (event: any) => {
        event.stopPropagation();

        try {
            if (isVisible) {
                folder.files = [];
                selectBucket({ ...bucket });
            } else {
                await getExpandedFolderFiles([...path, folder.name], folder, bucket);
            };

            setIsVisible(prev => !prev);
        } catch (error: any) { };
    };

    return (
        <>
            <tr
                className='cursor-pointer border-1 border-t-border-regular border-b-border-regular text-text-900 font-normal'
                onClick={event => goToFolder(event, bucket)}
            >
                <td
                    className="flex items-center gap-3 p-4"
                    style={{ paddingLeft: `${nestingLevel * 60}px` }}
                >
                    <span
                        className={`${!isVisible && 'rotate-180'} cursor-pointer p-2`}
                        onClick={expandFolder}
                    >
                        <ChevronUp />
                    </span>
                    <FileCell name={folder.name} />
                </td>
                <td className="px-6 py-4">{getDateLabel(+folder.metadata.modified)}</td>
                <td className="px-6 py-4">{convertFileSize(folder.metadata.size)}</td>
                <td className="px-6 py-4">
                    {
                        bucket.bucketType === 'backup' ?
                            null
                            :
                            <ActionsCell
                                actions={
                                    <FolderActions
                                        bucket={bucket}
                                        file={folder}
                                        parrentFolder={parrentFolder!}
                                        path={path}
                                    />
                                }
                                offsetTop={tableScroll}
                                tableRef={tableRef}
                            />
                    }
                </td>
            </tr>
            {isVisible &&
                folder.files?.map((file, index) =>
                    file.type === 'dir' ?
                        <FolderRow
                            bucket={bucket}
                            folder={file}
                            tableRef={tableRef}
                            tableScroll={tableScroll}
                            nestingLevel={nestingLevel + 1}
                            key={index}
                            parrentFolder={folder}
                            path={[...path, folder.name]}
                        />
                        :
                        <FileRow
                            bucket={bucket}
                            file={file}
                            tableRef={tableRef}
                            tableScroll={tableScroll}
                            nestingLevel={nestingLevel + 1}
                            parrentFolder={folder}
                            path={[...path, folder.name]}
                            key={index}
                        />
                )
            }
        </>
    )
}
