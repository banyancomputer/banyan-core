import React, { useState } from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { useModal } from '@contexts/modals';
import { Bucket, BucketSnapshot } from '@app/types/bucket';
import { useAppSelector } from '@app/store';
import { ToastNotifications } from '@app/utils/toastNotifications';

export const RenameSnapshotModal: React.FC<{ bucket: Bucket; snapshot: BucketSnapshot }> = ({ bucket, snapshot }) => {
    const { closeModal } = useModal();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.renameSnapshot);
    const [newName, setNewName] = useState('');

    const save = async () => {
        try {
            /** TODO: implement when backend will be ready */
            closeModal();
        } catch (error: any) {
            ToastNotifications.error(`${messages.editError}`);
        };
    };

    return (
        <div className="w-modal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.snapshotName}`}
                    <input
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-border-darken shadow-sm focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewName}`}
                        value={newName}
                        onChange={event => setNewName(event.target.value)}
                    />
                </label>
            </div>
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.save}`}
                    action={save}
                    disabled={newName.length < 3}
                />
            </div>
        </div >
    );
};
