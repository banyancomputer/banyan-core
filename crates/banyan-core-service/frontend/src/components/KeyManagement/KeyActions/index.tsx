import React from 'react';
import { useIntl } from 'react-intl';

import { ApproveBucketAccessModal } from '@/components/common/Modal/ApproveBucketAccessModal';
import { RemoveBucketAccessModal } from '@/components/common/Modal/RemoveBucketAccessModal';

import { Bucket, BucketKey } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';

export const KeyActions: React.FC<{ bucket: Bucket; bucketKey: BucketKey }> = ({ bucket, bucketKey }) => {
    const { messages } = useIntl();
    const { openModal } = useModal();

    const approveAccess = async() => {
        openModal(<ApproveBucketAccessModal bucketKey={bucketKey} />);
    };

    const removeAccess = async() => {
        try {
            openModal(<RemoveBucketAccessModal bucketKey={bucketKey} />);
        } catch (error: any) { }
    };

    return (
        <div className="relative w-52 text-xs font-medium bg-white rounded-xl shadow-md z-10 text-gray-900 overflow-hidden">
            {bucketKey.approved ?
                <div
                    className="w-full gap-2 py-2 px-3 border-b-1 border-gray-200 transition-all hover:bg-slate-200"
                    onClick={removeAccess}
                >
                    {`${messages.removeAccess}`}
                </div>
                :
                <div
                    className="w-full gap-2 py-2 px-3 border-b-1 border-gray-200 transition-all hover:bg-slate-200"
                    onClick={approveAccess}
                >
                    {`${messages.approveAccess}`}
                </div>
            }
        </div>
    );
};
