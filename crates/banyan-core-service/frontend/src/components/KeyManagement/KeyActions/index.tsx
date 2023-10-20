import React from 'react';
import { useIntl } from 'react-intl';

import { ApproveBucketAccessModal } from '@/components/common/Modal/ApproveBucketAccessModal';
import { RemoveBucketAccessModal } from '@/components/common/Modal/RemoveBucketAccessModal';

import { Bucket, BucketKey } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';

export const KeyActions: React.FC<{ bucket: Bucket; bucketKey: BucketKey }> = ({ bucket, bucketKey }) => {
    const { messages } = useIntl();
    const { openModal } = useModal();

    const approveAccess = async () => {
        openModal(<ApproveBucketAccessModal bucket={bucket} bucketKey={bucketKey} />);
    };

    const removeAccess = async () => {
        try {
            openModal(<RemoveBucketAccessModal bucketKey={bucketKey} />);
        } catch (error: any) { }
    };

    var approved = bucketKey.approved();

    return (
        <div className="w-52 text-xs font-medium bg-mainBackground rounded-xl shadow-md z-10 text-gray-900 overflow-hidden">
            {approved ?
                <div
                    className="w-full gap-2 py-2 px-3 border-b-1 border-border-regular transition-all hover:bg-hover"
                    onClick={removeAccess}
                >
                    {`${messages.removeAccess}`}
                </div>
                :
                <div
                    className="w-full gap-2 py-2 px-3 border-b-1 border-border-regular transition-all hover:bg-hover"
                    onClick={approveAccess}
                >
                    {`${messages.approveAccess}`}
                </div>
            }
        </div>
    );
};
