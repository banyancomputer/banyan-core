import React from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { closeModal } from '@store/modals/slice';
import { Bucket, BucketKey } from '@/app/types/bucket';
import { useAppDispatch, useAppSelector } from '@store/index';
import { removeBucketAccess } from '@store/tomb/actions';

export const RemoveBucketAccessModal: React.FC<{ bucket: Bucket; bucketKey: BucketKey }> = ({ bucket, bucketKey }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.removeBucketAccess);
    const dispatch = useAppDispatch();

    const close = () => {
        dispatch(closeModal());
    };

    const removeAccess = async () => {
        try {
            unwrapResult(await dispatch(removeBucketAccess({ bucket, bucketKeyId: bucketKey.id })));
            close();
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
                    action={close}
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
