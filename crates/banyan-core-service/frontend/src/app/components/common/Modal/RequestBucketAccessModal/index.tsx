import React from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { useModal } from '@contexts/modals';
import { Bucket } from '@app/types/bucket';
import { useAppSelector } from '@/app/store';

export const RequestBucketAccessModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.requestBucketAccess);
    const { closeModal } = useModal();

    const requestAccess = async () => {
        try {
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
                    text={`${messages.requestAccess}`}
                    action={requestAccess}
                />
            </div>
        </div>
    );
};
