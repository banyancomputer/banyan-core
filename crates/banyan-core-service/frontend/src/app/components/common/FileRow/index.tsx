import React, { useEffect, useRef, useState } from 'react';

import { ActionsCell } from '../ActionsCell';
import { FileActions } from '../FileActions';
import { FileCell } from '../FileCell';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { getDateLabel } from '@/app/utils/date';
import { convertFileSize } from '@/app/utils/storage';
import { useFilePreview } from '@/app/contexts/filesPreview';
import html2canvas from 'html2canvas';
import { DraggingPreview } from './DraggingPreview';

export const FileRow: React.FC<{
    file: BrowserObject;
    bucket: Bucket;
    tableScroll: number;
    tableRef: React.MutableRefObject<HTMLDivElement | null>;
    path: string[];
    nestingLevel?: number;
    parrentFolder?: BrowserObject;
}> = ({ file, bucket, tableScroll, tableRef, nestingLevel = 0.25, path = [], parrentFolder }) => {
    const { openFile } = useFilePreview();
    const [isDragging, setIsDragging] = useState(false);
    const [dragPreview, setDragPreview] = useState<null | Element>(null);
    const fileRowRef = useRef<HTMLElement | null>(null);

    const previewFile = (event: React.MouseEvent<HTMLTableRowElement, MouseEvent>, bucket: Bucket, file: BrowserObject) => {
        // @ts-ignore
        if (event.target.id === 'actionsCell') { return; }

        openFile(bucket, file.name, path);
    };

    const handleDragStart = async (event: React.DragEvent<HTMLDivElement>, file: BrowserObject) => {
        event.dataTransfer.setData('browserObject', JSON.stringify({ item: file, path }));
        event.dataTransfer.setDragImage(dragPreview!, 0, 0);
        setIsDragging(true);
    };

    const handleDragEnd = () => {
        setIsDragging(false);
    };

    useEffect(() => {
        if (!fileRowRef.current) return;
        (async () => {
            const preview = new Image();
            const canvas = await html2canvas(fileRowRef.current!);
            preview.src = canvas.toDataURL('image/jpg');
            setDragPreview(preview)
        })()
    }, [fileRowRef.current])

    return (
        <tr
            className={`cursor-pointer border-1 border-b-border-regular text-text-900 font-normal bg-secondaryBackground ${isDragging && 'opacity-50'} transition-all last:border-b-0 hover:bg-bucket-bucketHoverBackground ${isDragging && 'bg-slate-900'}`}
            onClick={event => previewFile(event, bucket, file)}
            onDragStart={event => handleDragStart(event, file)}
            onDragEnd={handleDragEnd}
            draggable
        >
            <td
                className="px-6 py-4"
                style={{ paddingLeft: `${nestingLevel * 60}px` }}
            >
                <span className="fixed -right-96" ref={fileRowRef}>
                    <DraggingPreview name={file.name} />
                </span>
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
