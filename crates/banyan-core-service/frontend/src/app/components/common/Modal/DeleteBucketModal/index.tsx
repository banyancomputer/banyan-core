import React from 'react';
import { useIntl } from 'react-intl';
import { FiTrash2 } from 'react-icons/fi';

import { Bucket } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

export const DeleteBucketModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { closeModal } = useModal();
    const { messages } = useIntl();
    const { deleteBucket } = useTomb();

    const removeBucket = async () => {
        try {
            await deleteBucket(bucket.id);
            closeModal();
            ToastNotifications.notify(`${messages.drive} "${bucket.name}" ${messages.wasDeleted}`, <FiTrash2 size="20px" />);
        } catch (error: any) {
            ToastNotifications.error(`${messages.deletionError}`, `${messages.tryAgain}`, removeBucket);
            closeModal();
        };
    };

    return (
        <div className="w-modal flex flex-col gap-5">
            <FiTrash2 size="24px" stroke="#5e6c97" />
            <div>
                <h4 className="text-m font-semibold">{`${messages.deleteBucket}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.wantToEmpty}`} <b className="text-text-900">{bucket.name}</b>? {`${messages.filesWillBeDeletedPermanently}`}.
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
                    onClick={removeBucket}
                >
                    {`${messages.delete}`}
                </button>
            </div>
        </div>
    );
};