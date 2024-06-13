import { useEffect } from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { useAppDispatch, useAppSelector } from '@app/store';
import { ToastNotifications } from '@utils/toastNotifications';
import { getDateLabel, getTime } from '@utils/date';
import { convertFileSize } from '@utils/storage';

import { Bucket } from '@/app/types/bucket';
import { getSelectedBucketSnapshots, restore } from '@store/tomb/actions';

export const SnapshotsTable = () => {
    const dispatch = useAppDispatch();
    const { tomb, selectedBucket } = useAppSelector(state => state.tomb);
    const { date, size, state } = useAppSelector(state => state.locales.messages.coponents.bucket.snapshots.table);
    const snapshotsActionsMessages = useAppSelector(state => state.locales.messages.coponents.bucket.snapshots.table.snapshotActions);

    const restoreFromSnapshot = async (bucket: Bucket, snapshotId: string) => {
        try {
            unwrapResult(await dispatch(restore({ bucket, snapshotId })));
            ToastNotifications.notify('Restoring could take up to 72 hours');
        } catch (error: any) {
            ToastNotifications.error('Error while restoring from snapshot');
        };
    };

    useEffect(() => {
        if (!tomb || !selectedBucket) { return; }

        (async () => {
            try {
                unwrapResult(await dispatch(getSelectedBucketSnapshots(selectedBucket?.id || '')));
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
                        <th className="px-6 py-4 text-left font-semibold w-56">
                            {date}
                        </th>
                        <th className="px-6 py-4 text-left font-semibold w-36  ">
                            {size}
                        </th>
                        <th className="px-6 py-4 w-20 text-xs text-left font-semibold">
                            {state}
                        </th>
                        <th className="px-6 py-4 w-20 text-xs text-left font-semibold" />
                    </tr>
                </thead>
                <tbody>
                    {selectedBucket?.snapshots.map(snapshot =>
                        <tr
                            className="cursor-pointer border-b-1 border-b-border-regular text-text-900 font-normal transition-all last:border-b-0 hover:bg-bucket-bucketHoverBackground"
                        >
                            <td className="px-6 py-2 whitespace-nowrap overflow-hidden text-ellipsis">{`${getDateLabel(snapshot.created_at)}, ${getTime(snapshot.created_at)}`}</td>
                            <td className="px-6 py-2 font-semibold">{convertFileSize(snapshot.size)}</td>
                            <td className="px-6 py-2 capitalize">{snapshot.state}</td>
                            <td className="px-6 py-2 font-semibold text-right text-button-primary transition-all hover:text-button-primaryHover">
                                <button onClick={() => restoreFromSnapshot(selectedBucket, snapshot.metadata_id)}>
                                    {snapshotsActionsMessages.restore}
                                </button>
                            </td>
                        </tr>
                    )}
                </tbody>
            </table>
        </div>
    )
}
