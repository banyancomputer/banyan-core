import React from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { closeModal } from '@store/modals/slice';
import { Bucket, BucketKey } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppDispatch, useAppSelector } from '@store/index';
import { approveBucketAccess } from '@store/tomb/actions';
import { unwrapResult } from '@reduxjs/toolkit';

export const ApproveBucketAccessModal: React.FC<{ bucket: Bucket; bucketKey: BucketKey }> = ({ bucket, bucketKey }) => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.approveBucketAccess);

    const cancel = () => {
        dispatch(closeModal());
    };

    const approveAccess = async () => {
        try {
            unwrapResult(await dispatch(approveBucketAccess({ bucket, bucketKeyId: bucketKey.id })));
            cancel();
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
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={cancel}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.approveAccess}`}
                    action={approveAccess}
                />
            </div>
        </div>
    );
};
