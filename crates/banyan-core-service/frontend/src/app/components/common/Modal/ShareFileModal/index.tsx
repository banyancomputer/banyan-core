import React from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { useModal } from '@/app/contexts/modals';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppSelector } from '@/app/store';

export const ShareFileModal: React.FC<{ link: string }> = ({ link }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.shareFile);
    const { closeModal } = useModal();

    const copy = () => {
        navigator.clipboard?.writeText(link);
        closeModal();
        ToastNotifications.notify(`${messages.linkWasCopied}`);
    };

    return (
        <div className="w-modal flex flex-col gap-7" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
            </div>
            <div className="flex flex-col items-start gap-3">
                <div className="w-full overflow-hidden text-ellipsis whitespace-nowrap text-xs">{link}</div>
            </div>
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.copyLink}`}
                    action={copy}
                />
            </div>
        </div >
    );
};
