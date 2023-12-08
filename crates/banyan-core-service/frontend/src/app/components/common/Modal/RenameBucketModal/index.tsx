import React, { useState } from 'react';
import { useIntl } from 'react-intl';

import { SubmitButton } from '@components/common/SubmitButton';

import { useModal } from '@/app/contexts/modals';
import { Bucket } from '@/app/types/bucket';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

import { Done } from '@static/images/common';

export const RenameBucketModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { closeModal } = useModal();
    const { messages } = useIntl();
    const [newName, setNewName] = useState('');
    const { } = useTomb();

    const rename = async () => {
        try {
            /** TODO: add rename function after it will be added into tomb-wasm */
            ToastNotifications.notify(`${messages.drive} "${bucket.name}" ${messages.wasRenamed}`, <Done width="20px" height="20px" />);
        } catch (error: any) { };
    };

    return (
        <div className="w-modal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.renameDrive}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.driveName}`}
                    <input
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-lg border-border-darken focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewDriveName}`}
                        value={newName}
                        onChange={event => setNewName(event.target.value)}
                    />
                </label>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <SubmitButton
                    text={`${messages.save}`}
                    action={rename}
                    disabled={newName.length < 3}
                />
            </div>
        </div >
    );
};
