import React, { useState } from 'react'
import { useIntl } from 'react-intl';
import { BiShareAlt } from 'react-icons/bi'

import { useTomb } from '@/contexts/tomb'
import { useModal } from '@/contexts/modals';
import { Bucket, BucketFile } from '@/lib/interfaces/bucket';
import { ToastNotifications } from '@/utils/toastNotifications';

export const ShareWithModal: React.FC<{ bucket: Bucket, file: BucketFile }> = ({ bucket, file }) => {
    const { shareWith } = useTomb();
    const [email, setEmail] = useState('');
    const { messages } = useIntl();
    const { closeModal } = useModal();

    const cancel = () => {
        closeModal();
        ToastNotifications.notify(
            `${messages.requestCanceled}`,
            <BiShareAlt size="20px" />
        );
    };

    const share = async () => {
        try {
            await shareWith(bucket, email);
            ToastNotifications.notify(`${messages.yourFile} ${file.name} ${messages.hasBeenShared} ${email}`);
        } catch (error: any) {
            console.log(error);
        };
    };

    return (
        <div className="w-modal flex flex-col gap-2" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.shareWith}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.recipientEmail}`}
                    <input
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-lg border-gray-300 shadow-sm focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterEmail}`}
                        value={email}
                        onChange={event => setEmail(event.target.value)}
                    />
                </label>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={cancel}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary flex-grow py-3 px-4"
                    onClick={share}
                >{`${messages.share}`}</button>
            </div>
        </div >
    )
};
