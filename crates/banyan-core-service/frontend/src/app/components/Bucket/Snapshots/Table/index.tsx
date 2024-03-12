import React, { useEffect, useState } from 'react';

import { SortCell } from '@components/common/SortCell';

import { useTomb } from '@app/contexts/tomb';
import { useAppSelector } from '@app/store';
import { ToastNotifications } from '@app/utils/toastNotifications';
import { getDateLabel, getTime } from '@app/utils/date';
import { convertFileSize } from '@app/utils/storage';
import { Directory } from '@/app/static/images/common';
import { ActionsCell } from '@/app/components/common/ActionsCell';
import { SnapshotActions } from './SnapshotActions';

export const SnapshotsTable = () => {
    const { getBucketShapshots, tomb, selectedBucket } = useTomb();
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.snapshots.table);
    const [sortState, setSortState] = useState<{ criteria: string; direction: 'ASC' | 'DESC' | '' }>({ criteria: 'name', direction: 'DESC' });

    const sort = (criteria: string) => {
        setSortState(prev => ({ criteria, direction: prev.direction === 'ASC' ? 'DESC' : 'ASC' }));
    };

    useEffect(() => {
        if (!tomb || !selectedBucket) { return; }

        (async () => {
            try {
                await getBucketShapshots(selectedBucket?.id || '');
            } catch (error: any) {
                ToastNotifications.error('Error while getting snapshots');
            }
        })();
    }, [tomb, selectedBucket]);

    return (
        <div
            className="w-fit h-full max-h-[calc(100vh-388px)] px-6 py-0 text-xs"
            id="table"
        >
            <table
                className="table table-pin-rows w-full text-text-600 rounded-xl table-fixed"
            >
                <thead className="border-b-border-regular border-b-2 font-normal text-text-900">
                    <tr className="bg-mainBackground font-normal border-none">
                        <th className="flex items-center gap-3 pl-2 py-4 text-left font-medium">
                            <SortCell
                                criteria="name"
                                onChange={sort}
                                sortState={sortState}
                                text={messages.name}
                            />
                        </th>
                        <th className="px-6 py-4 text-left font-semibold w-56">
                            <SortCell
                                criteria="date"
                                onChange={sort}
                                sortState={sortState}
                                text={messages.date}
                            />
                        </th>
                        <th className="px-6 py-4 text-left font-semibold w-36  ">
                            <SortCell
                                criteria="size"
                                onChange={sort}
                                sortState={sortState}
                                text={messages.size}
                            />
                        </th>
                        <th className="px-6 py-4 w-20 text-xs text-left font-semibold">
                            {messages.state}
                        </th>
                        <th className="px-6 py-4 w-20 text-xs text-left font-semibold" />
                    </tr>
                </thead>
                <tbody>
                    {selectedBucket?.snapshots.map(snapshot =>
                        <tr
                            className="cursor-pointer border-b-1 border-b-border-regular text-text-900 font-normal transition-all last:border-b-0 hover:bg-bucket-bucketHoverBackground"
                        >
                            <td className="pr-6 pl-2 py-2">
                                <div className="flex gap-2">
                                    <Directory width="20px" height="20px" />
                                </div>
                            </td>
                            <td className="px-6 py-2 whitespace-nowrap overflow-hidden text-ellipsis">{`${getDateLabel(snapshot.createdAt)}, ${getTime(snapshot.createdAt)}`}</td>
                            <td className="px-6 py-2 font-semibold">{convertFileSize(snapshot.size)}</td>
                            <td className="px-6 py-2">{snapshot.snapshot_type}</td>
                            <td className="px-6 py-2">
                                <ActionsCell
                                    actions={<SnapshotActions bucket={selectedBucket} snapshot={snapshot} />}
                                />
                            </td>
                        </tr>
                    )}
                </tbody>
            </table>
        </div>
    )
}
