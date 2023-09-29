import React, { useEffect, useState } from 'react';
import { useIntl } from 'react-intl';
import { AiOutlineFile } from 'react-icons/ai';
import { FiDownload } from 'react-icons/fi';

import { useTomb } from '@/contexts/tomb';
import { BucketSnapshot } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';
import { getDateLabel } from '@/utils/date';

export const BucketSnapshotsModal: React.FC<{ bucketId: string }> = ({ bucketId }) => {
    const { getBucketShapshots, tomb } = useTomb();
    const { closeModal } = useModal();
    const [snapshots, setSnapshots] = useState<BucketSnapshot[]>([]);
    const { messages } = useIntl();

    useEffect(() => {
        if (!tomb) { return; }

        (async() => {
            try {
                const snapshots = await getBucketShapshots(bucketId);
                setSnapshots(snapshots);
            } catch (error: any) {
                console.log(error);
            }
        })();
    }, [tomb]);

    return (
        <div className="w-snapshotsModal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.viewColdSnapshots}`}</h4>
                <p className="mt-2 text-gray-600">
                    {`${messages.accessPreviousVersions}`}
                </p>
            </div>
            <div className="flex flex-col gap-3">
                {snapshots.map(snapshot =>
                    <div
                        className="flex align-middle gap-3 px-3 py-2  border-1 border-navigation-border rounded-xl text-xs"
                        key={snapshot.id}
                    >
                        <div className="flex align-middle gap-2 flex-grow">
                            <AiOutlineFile size="20px" />
                            <span className="font-semibold">
                                {snapshot.id}
                            </span>
                        </div>
                        <div>
                            Size: {snapshot.size}, Created: {`${new Date(snapshot.createdAt * 1000)}`}
                        </div>
                    </div>
                )}
            </div>
            <button
                className="btn-primary py-2 text-xs"
                onClick={closeModal}
            >
                {`${messages.close}`}
            </button>
        </div>
    );
};
