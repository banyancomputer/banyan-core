import React from 'react';
import { useIntl } from 'react-intl';

import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { Bucket, BucketKey } from '@/app/types/bucket';

export const ApproveBucketAccessModal: React.FC<{ bucket: Bucket; bucketKey: BucketKey }> = ({ bucket, bucketKey }) => {
    const { messages } = useIntl();
    const { approveBucketAccess } = useTomb();
    const { closeModal } = useModal();

    const approveAccess = async () => {
        try {
            await approveBucketAccess(bucket, bucketKey.id);
            closeModal();
        } catch (error: any) { }
    };

    return (
        <div className="w-modal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.approveAccess}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.wantToApproveAccess}?`}
                </p>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary w-1/2 py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary w-1/2 py-3 px-4"
                    onClick={approveAccess}
                >{`${messages.approveAccess}`}</button>
            </div>
        </div>
    );
};
