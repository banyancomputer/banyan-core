import { useIntl } from 'react-intl';

import { SecondaryButton } from '@components/common/SecondaryButton';
import { PrimaryButton } from '@components/common/PrimaryButton';

import { useModal } from '@/app/contexts/modals';
import { ToastNotifications } from '@/app/utils/toastNotifications';

import { Trash } from '@static/images/common';

export const EmptyTrashModal = () => {
    const { closeModal } = useModal();
    const { messages } = useIntl();

    const clearTrash = async () => {
        ToastNotifications.notify(`${messages.trashWasCleaned}`, <Trash width="20px" height="20px" />);
    };

    return (
        <div className="w-modal flex flex-col gap-5">
            <Trash width="24px" height="24px" />
            <div>
                <h4 className="text-m font-semibold">{`${messages.deleteBucket}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.wantToEmpty}`} <b className="text-text-900">{`${messages.trash}`}</b>? {`${messages.filesWillBeDeletedPermanently}`}.
                </p>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.delete}`}
                    action={clearTrash}
                />
            </div>
        </div>
    );
};
