import React from 'react';
import { useIntl } from 'react-intl';
import { HiOutlineLightningBolt } from 'react-icons/hi';

import { Bucket } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';
import { useTomb } from '@/contexts/tomb';
import { ToastNotifications } from '@/utils/toastNotifications';

export const TakeSnapshotModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { closeModal } = useModal();
    const { takeColdSnapshot } = useTomb();
    const { messages } = useIntl();

    const takeSnapshot = async() => {
        try {
            await takeColdSnapshot(bucket);
            ToastNotifications.notify(`${messages.snapshotWasTaken}`, <HiOutlineLightningBolt size="20px" />);
            closeModal();
        } catch (error: any) {
            closeModal();
            ToastNotifications.error(`${messages.snapshotError}`, `${messages.tryAgain}`, takeSnapshot);
        }
    };

    return (
        <div className="w-modal flex flex-col gap-5">
            <span className="p-3 w-min rounded-full bg-gray-200">
                <HiOutlineLightningBolt size="24px" />
            </span>
            <div>
                <h4 className="text-m font-semibold">{`${messages.takeColdSnapshot}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.doYouWantToTakeSnapshot}`}?
                </p>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary w-1/2 py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary w-1/2 py-3 px-4"
                    onClick={takeSnapshot}
                >
                    {`${messages.takeColdSnapshot}`}
                </button>
            </div>
        </div>
    );
};
