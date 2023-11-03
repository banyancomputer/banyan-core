import React from 'react';
import { useIntl } from 'react-intl';

import { useModal } from '@/app/contexts/modals';
import { ToastNotifications } from '@/app/utils/toastNotifications';

export const ShareFileModal: React.FC<{ link: string }> = ({ link }) => {
    const { messages } = useIntl();
    const { closeModal } = useModal();

    const copy = () => {
        navigator.clipboard?.writeText(link);
        closeModal();
        ToastNotifications.notify(`${messages.linkWasCopied}`);
    };

    return (
        <div className="w-modal flex flex-col gap-7" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.share}`}</h4>
            </div>
            <div className="flex flex-col items-start gap-3">
                <span className="text-xs">{link}</span>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary flex-grow py-3 px-4"
                    onClick={copy}
                >{`${messages.copyLink}`}</button>
            </div>
        </div >
    );
};
