import React from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { Bucket } from '@/app/types/bucket';
import { closeModal } from '@store/modals/slice';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppDispatch, useAppSelector } from '@store/index';

import { Trash } from '@static/images/common';
import { deleteBucket } from '@store/tomb/actions';
import { unwrapResult } from '@reduxjs/toolkit';

export const DeleteDriveModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.deleteBucket);

    const close = () => {
        dispatch(closeModal());
    };

    const removeBucket = async () => {
        try {
            unwrapResult(await dispatch(deleteBucket(bucket.id)));
            dispatch(closeModal());
            ToastNotifications.notify(`${messages.drive} "${bucket.name}" ${messages.wasDeleted}`, <Trash width="20px" height="20px" />);
        } catch (error: any) {
            ToastNotifications.error(`${messages.deletionError}`, `${messages.tryAgain}`, removeBucket);
            dispatch(closeModal());
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
                    action={close}
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
