import React, { useState } from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { useModal } from '@/app/contexts/modals';
import { Bucket } from '@/app/types/bucket';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppSelector } from '@/app/store';

import { Done } from '@static/images/common';

export const RenameBucketModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { closeModal } = useModal();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.renameBucket);
    const [newName, setNewName] = useState('');
    const { renameBucket } = useTomb();

    const rename = async () => {
        try {
            await renameBucket(bucket, newName);
            closeModal();
            ToastNotifications.notify(`${messages.drive} "${bucket.name}" ${messages.wasRenamed}`, <Done width="20px" height="20px" />);
        } catch (error: any) {
            closeModal();
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
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
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
