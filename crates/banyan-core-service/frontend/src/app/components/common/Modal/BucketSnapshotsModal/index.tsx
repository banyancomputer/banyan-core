import React, { useEffect, useState } from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';

import { useTomb } from '@/app/contexts/tomb';
import { BucketSnapshot } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';
import { getDateLabel, getTime } from '@/app/utils/date';
import { convertFileSize } from '@/app/utils/storage';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppSelector } from '@/app/store';

import { CommonFileIcon } from '@static/images/common';

export const BucketSnapshotsModal: React.FC<{ bucketId: string }> = ({ bucketId }) => {
    const { getBucketShapshots, tomb } = useTomb();
    const { closeModal } = useModal();
    const [snapshots, setSnapshots] = useState<BucketSnapshot[]>([]);
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.bucketSnapshots);

    useEffect(() => {
        if (!tomb) { return; }

        const getSnapshots = async () => {
            try {
                const snapshots = await getBucketShapshots(bucketId);
                setSnapshots(snapshots);
            } catch (error: any) {
                ToastNotifications.error('Error while getting snapshots', `${messages.tryAgain}`, getSnapshots);
                console.log(error);
            }
        };

        getSnapshots();
    }, [tomb]);

    return (
        <div className="w-snapshotsModal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
                <p className="mt-2 text-gray-400">
                    {`${messages.subtitle}`}
                </p>
            </div>
            <div className="flex flex-col gap-3">
                {snapshots.map(snapshot =>
                    <div
                        className="flex align-middle gap-3 px-3 py-2  border-1 border-gray-200 rounded-xl text-xs"
                        key={snapshot.id}
                    >
                        <div className="flex items-center align-middle gap-2 flex-grow text-navigation-border">
                            <CommonFileIcon width="20px" height="20px" />
                            <div className="flex flex-col font-semibold text-text-900">
                                <span>{`${getDateLabel(snapshot.createdAt, false)} version`}</span>
                                <span className="text-gray-400 font-medium">{`${convertFileSize(snapshot.size)}`}</span>
                            </div>
                        </div>
                        <div className="flex items-center whitespace-nowrap font-medium">
                            {`${getDateLabel(snapshot.createdAt)}, ${getTime(snapshot.createdAt)}`}
                        </div>
                    </div>
                )}
            </div>
            <PrimaryButton
                text={`${messages.close}`}
                action={closeModal}
                className="w-full py-2 text-xxs"
            />
        </div>
    );
};
