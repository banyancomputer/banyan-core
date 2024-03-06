import React from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { BucketKey } from '@/app/types/bucket';
import { useAppSelector } from '@/app/store';

export const RemoveBucketAccessModal: React.FC<{ bucketKey: BucketKey }> = ({ bucketKey }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.removeBucketAccess);
    const { removeBucketAccess } = useTomb();
    const { closeModal } = useModal();

    const removeAccess = async () => {
        try {
            await removeBucketAccess(bucketKey.id);
            closeModal();
        } catch (error: any) { }
    };

    return (
        <div className="w-modal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.subtitle}`}
                </p>
            </div>
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.removeAccess}`}
                    action={removeAccess}
                />
            </div>
        </div>
    );
};
