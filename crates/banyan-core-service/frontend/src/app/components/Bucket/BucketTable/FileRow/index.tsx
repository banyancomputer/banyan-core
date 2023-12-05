import React, { useState } from 'react';

import { ActionsCell } from '@components/common/ActionsCell';
import { FileActions } from '../FileActions';
import { FileCell } from '@components/common/FileCell';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { getDateLabel } from '@/app/utils/date';
import { convertFileSize } from '@/app/utils/storage';
import { useFilePreview } from '@/app/contexts/filesPreview';
import { DraggingPreview } from './DraggingPreview';
import { handleDrag, handleDragEnd, handleDragStart } from '@app/utils/dragHandlers';

export const FileRow: React.FC<{
    file: BrowserObject;
    bucket: Bucket;
    tableScroll: number;
    tableRef: React.MutableRefObject<HTMLDivElement | null>;
    path: string[];
    siblingFiles: string[];
    nestingLevel?: number;
    parrentFolder?: BrowserObject;
}> = ({ file, bucket, tableScroll, tableRef, nestingLevel = 0.25, path = [], parrentFolder, siblingFiles }) => {
    const { openFile } = useFilePreview();
    const [isDragging, setIsDragging] = useState(false);

    const previewFile = (event: React.MouseEvent<HTMLTableRowElement, MouseEvent>, bucket: Bucket, file: BrowserObject) => {
        // @ts-ignore
        if (event.target.id === 'actionsCell') { return; }

        openFile(bucket, file.name, siblingFiles, path);
    };

    return (
        <tr
            className={`cursor-pointer border-b-1 border-b-border-regular text-text-900 font-normal transition-all last:border-b-0 hover:bg-bucket-bucketHoverBackground`}
            onClick={event => previewFile(event, bucket, file)}
            onDrag={event => handleDrag(event, file.name)}
            onDragStart={event => handleDragStart(event, file, setIsDragging, path)}
            onDragEnd={() => handleDragEnd(setIsDragging)}
            draggable
        >
            <td
                className="px-6 py-4"
                style={{ paddingLeft: `${nestingLevel * 60}px` }}
            >
                <DraggingPreview name={file.name} isDragging={isDragging} />
                <span>
                    <FileCell name={file.name} />
                </span>
            </td>
            <td className="px-6 py-4">{getDateLabel(+file.metadata.modified)}</td>
            <td className="px-6 py-4">{convertFileSize(file.metadata.size)}</td>
            <td className="px-6 py-4">
                <ActionsCell
                    actions={
                        <FileActions
                            bucket={bucket}
                            file={file}
                            parrentFolder={parrentFolder!}
                            path={path}
                        />
                    }
                    offsetTop={tableScroll}
                    tableRef={tableRef}
                />
            </td>
        </tr>
    );
};
