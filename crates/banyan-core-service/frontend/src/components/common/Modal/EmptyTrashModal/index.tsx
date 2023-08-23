import React from 'react';
import { useIntl } from 'react-intl';
import { FiTrash2 } from 'react-icons/fi';

import { useModal } from '@/contexts/modals';
import { ToastNotifications } from '@/utils/toastNotifications';

export const EmptyTrashModal = () => {
    const { closeModal } = useModal();
    const { messages } = useIntl();

    const clearTrash = async() => {
        ToastNotifications.notify(`${messages.trashWasCleaned}`, <FiTrash2 size="20px" />);
    };

    return (
        <div className="w-modal flex flex-col gap-5">
            <FiTrash2 size="24px" stroke="#5e6c97" />
            <div>
                <h4 className="text-m font-semibold">{`${messages.deleteBucket}`}</h4>
                <p className="mt-2 text-gray-600">
                    {`${messages.wantToEmpty}`} <b className="text-gray-900">{`${messages.trash}`}</b>? {`${messages.filesWillBeDeletedPermanently}`}.
                </p>
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
                    onClick={clearTrash}
                >
                    {`${messages.delete}`}
                </button>
            </div>
        </div>
    );
};
