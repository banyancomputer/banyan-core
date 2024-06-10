import React from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { Bucket } from '@/app/types/bucket';
import { closeModal } from '@store/modals/slice';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppDispatch, useAppSelector } from '@store/index';
import { takeColdSnapshot, updateStorageUsageState } from '@store/tomb/actions';

import { Bolt } from '@static/images/common';

export const TakeSnapshotModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.takeSnapshot);
    const dispatch = useAppDispatch();

    const close = () => {
        dispatch(closeModal());
    };

    const takeSnapshot = async () => {
        try {
            unwrapResult(await dispatch(takeColdSnapshot(bucket)));
            unwrapResult(await dispatch(updateStorageUsageState()));
            ToastNotifications.notify(`${messages.snapshotWasTaken}`, <Bolt width="20px" height="20px" />);
            close();
        } catch (error: any) {
            close();
            ToastNotifications.error(`${messages.snapshotError}`, `${messages.tryAgain}`, takeSnapshot);
        }
    };

    return (
        <div className="w-takeSnapshotModal flex flex-col gap-5">
            <div>
                <h4 className="text-m font-semibold">{`${messages.title}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.subtitle}`}?
                </p>
            </div>
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={close}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.takeArchivalSnapshot}`}
                    action={takeSnapshot}
                />
            </div>
        </div>
    );
};
