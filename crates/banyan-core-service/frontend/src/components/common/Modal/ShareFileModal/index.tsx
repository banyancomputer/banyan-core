import React from 'react'
import { useIntl } from 'react-intl';
import { BiLinkAlt, BiShareAlt } from 'react-icons/bi'

import { useModal } from '@/contexts/modals';
import { ToastNotifications } from '@/utils/toastNotifications';

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
            <div className='flex flex-col items-start gap-3'>
                <span className='text-xs'>{link}</span>
                <button
                    className='flex items-center gap-3 underline text-xs font-medium'
                    onClick={copy}
                >
                    <span className='text-blue-primary'>
                        <BiLinkAlt size="14px" />
                    </span>
                    {`${messages.copyLink}`}
                </button>
            </div>
        </div >
    )
};
