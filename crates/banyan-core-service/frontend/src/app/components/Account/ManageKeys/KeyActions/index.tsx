import React from 'react';

import { ApproveBucketAccessModal } from '@components/common/Modal/ApproveBucketAccessModal';
import { RemoveBucketAccessModal } from '@components/common/Modal/RemoveBucketAccessModal';

import { Bucket, BucketKey } from '@app/types/bucket';
import { useModal } from '@app/contexts/modals';
import { useAppSelector } from '@/app/store';

export const KeyActions: React.FC<{ bucket: Bucket; bucketKey: BucketKey }> = ({ bucket, bucketKey }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.account.manageKeys.keyActions);
    const { openModal } = useModal();

    const approveAccess = async () => {
        openModal(<ApproveBucketAccessModal bucket={bucket} bucketKey={bucketKey} />);
    };

    const removeAccess = async () => {
        openModal(<RemoveBucketAccessModal bucketKey={bucketKey} />);
    };

    return (
        <div className="w-52 text-xs font-medium bg-bucket-actionsBackground rounded-xl shadow-md z-10 text-bucket-actionsText overflow-hidden">
            {bucketKey.approved ?
                <div
                    className="w-full gap-2 py-2 px-3 border-b-1 border-border-regular transition-all hover:bg-hover"
                    onClick={removeAccess}
                >
                    {messages.removeAccess}
                </div>
                :
                <div
                    className="w-full gap-2 py-2 px-3 border-b-1 border-border-regular transition-all hover:bg-hover"
                    onClick={approveAccess}
                >
                    {messages.approveAccess}
                </div>
            }
        </div>
    );
};
