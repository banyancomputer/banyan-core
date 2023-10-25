import React from 'react';

import { ActionsCell } from '../ActionsCell';
import { FileActions } from '../FileActions';
import { FileCell } from '../FileCell';

import { BrowserObject, Bucket } from '@/lib/interfaces/bucket';
import { getDateLabel } from '@/utils/date';
import { convertFileSize } from '@/utils/storage';
import { useFilePreview } from '@/contexts/filesPreview';

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

    const previewFile = (event: React.MouseEvent<HTMLTableRowElement, MouseEvent>, bucket: Bucket, file: BrowserObject) => {
        // @ts-ignore
        if (event.target.id === 'actionsCell') { return; }

        openFile(bucket, file.name, path);
    };

    return (
        <tr
            className="cursor-pointer border-1 border-t-border-regular border-b-border-regular text-text-900 font-normal"
            onClick={event => previewFile(event, bucket, file)}
        >
            <td
                className="px-6 py-4"
                style={{ paddingLeft: `${nestingLevel * 60}px` }}
            >
                <FileCell name={file.name} />
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
