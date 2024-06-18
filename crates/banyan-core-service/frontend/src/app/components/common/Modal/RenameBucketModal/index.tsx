import React, { useState } from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { closeModal } from '@store/modals/slice';
import { Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppDispatch, useAppSelector } from '@store/index';

import { Done } from '@static/images/common';
import { renameBucket } from '@store/tomb/actions';

export const RenameBucketModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.renameBucket);
    const [newName, setNewName] = useState('');

    const close = () => {
        dispatch(closeModal());
    };

    const rename = async () => {
        try {
            unwrapResult(await dispatch(renameBucket({ bucket, name: newName })));
            close();
            ToastNotifications.notify(`${messages.drive} "${bucket.name}" ${messages.wasRenamed}`, <Done width="20px" height="20px" />);
        } catch (error: any) {
            close();
            ToastNotifications.error(`${messages.editError}`, `${messages.tryAgain}`, rename);
        };
    };

    return (
        <div className="w-modal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.enterNewName}`}
                    <input
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-border-darken focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewName}`}
                        value={newName}
                        onChange={event => setNewName(event.target.value)}
                    />
                </label>
            </div>
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={close}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.save}`}
                    action={rename}
                    disabled={newName.length < 3}
                />
            </div>
        </div >
    );
};
