import React from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { Bucket } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppSelector } from '@/app/store';

import { Trash } from '@static/images/common';

export const DeleteDriveModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.deleteBucket);
    const { closeModal } = useModal();
    const { deleteBucket } = useTomb();

    const removeBucket = async () => {
        try {
            await deleteBucket(bucket.id);
            closeModal();
            ToastNotifications.notify(`${messages.drive} "${bucket.name}" ${messages.wasDeleted}`, <Trash width="20px" height="20px" />);
        } catch (error: any) {
            ToastNotifications.error(`${messages.deletionError}`, `${messages.tryAgain}`, removeBucket);
            closeModal();
        };
    };

    return (
        <div className="w-modal flex flex-col gap-5">
            <div>
                <h4 className="text-m font-semibold">{`${messages.title}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.subtitle}`} <b className="text-text-900">{bucket.name}</b>? {`${messages.filesWillBeDeletedPermanently}`}.
                </p>
            </div>
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.delete}`}
                    action={removeBucket}
                />
            </div>
        </div>
    );
};
