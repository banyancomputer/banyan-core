import React, { useState } from 'react';

import { ActionsCell } from '@components/common/ActionsCell';
import { FileActions } from '../FileActions';
import { FileCell } from '@components/common/FileCell';
import { DraggingPreview } from './DraggingPreview';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { getDateLabel } from '@/app/utils/date';
import { convertFileSize } from '@/app/utils/storage';
import { handleDrag, handleDragEnd, handleDragStart } from '@utils/dragHandlers';
import { openFile } from '@store/filePreview/slice';
import { useAppDispatch } from '@store/index';

export const FileRow: React.FC<{
    file: BrowserObject;
    bucket: Bucket;
    path: string[];
    siblingFiles: BrowserObject[];
    nestingLevel?: number;
    parrentFolder?: BrowserObject;
}> = ({ file, bucket, nestingLevel = 0, path = [], parrentFolder, siblingFiles }) => {
    const [isDragging, setIsDragging] = useState(false);
    const dispatch = useAppDispatch();

    const previewFile = (event: React.MouseEvent<HTMLTableRowElement, MouseEvent>, bucket: Bucket, file: BrowserObject) => {
        //@ts-ignore
        if (event.target.id === 'actionsCell') {
            return;
        };

        dispatch(openFile({ bucket, file, files: siblingFiles, path, parrentFolder }));
    };

    return (
        <tr
            className="cursor-pointer border-b-1 border-b-border-regular text-text-900 font-normal transition-all last:border-b-0 hover:bg-bucket-bucketHoverBackground"
            onClick={event => previewFile(event, bucket, file)}
            onDrag={event => handleDrag(event, file.name)}
            onDragStart={event => handleDragStart(event, file.name, setIsDragging, path)}
            onDragEnd={() => handleDragEnd(setIsDragging)}
            draggable
        >
            <td
                className="px-0 py-2"
                style={{ paddingLeft: `${nestingLevel * 40}px` }}
            >
                <DraggingPreview browserObject={file} isDragging={isDragging} />
                <span>
                    <FileCell borwserObject={file} />
                </span>
            </td>
            <td className="px-6 py-2">{getDateLabel(+file.metadata.modified)}</td>
            <td className="px-6 py-2">{convertFileSize(file.metadata.size)}</td>
            <td className="px-6 py-0">
                <ActionsCell
                    actions={
                        <FileActions
                            bucket={bucket}
                            file={file}
                            parrentFolder={parrentFolder!}
                            path={path}
                        />
                    }
                />
            </td>
        </tr>
    );
};
