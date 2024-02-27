import React from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { Bucket, BucketKey } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppSelector } from '@/app/store';

export const ApproveBucketAccessModal: React.FC<{ bucket: Bucket; bucketKey: BucketKey }> = ({ bucket, bucketKey }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.approveBucketAccess);
    const { approveBucketAccess } = useTomb();
    const { closeModal } = useModal();

    const approveAccess = async () => {
        try {
            await approveBucketAccess(bucket, bucketKey.id);
            closeModal();
        } catch (error: any) {
            ToastNotifications.error('Something went wrong', `${messages.tryAgain}`, approveAccess);
        }
    };

    return (
        <div className="w-modal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.subtitle}?`}
                </p>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <SecondaryButton
                    className="w-1/2"
                    action={closeModal}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.approveAccess}`}
                    action={approveAccess}
                    className="w-1/2"
                />
            </div>
        </div>
    );
};
