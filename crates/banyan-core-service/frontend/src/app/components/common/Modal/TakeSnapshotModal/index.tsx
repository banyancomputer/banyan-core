import React from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { Bucket } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppSelector } from '@/app/store';

import { Bolt } from '@static/images/common';

export const TakeSnapshotModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.takeSnapshot);
    const { closeModal } = useModal();
    const { takeColdSnapshot } = useTomb();

    const takeSnapshot = async () => {
        try {
            await takeColdSnapshot(bucket);
            ToastNotifications.notify(`${messages.snapshotWasTaken}`, <Bolt width="20px" height="20px" />);
            closeModal();
        } catch (error: any) {
            closeModal();
            ToastNotifications.error(`${messages.snapshotError}`, `${messages.tryAgain}`, takeSnapshot);
        }
    };

    return (
        <div className="w-takeSnapshotModal flex flex-col gap-5">
            <div>
                <h4 className="text-m font-semibold">{`${messages.title}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.subtitle}`}?
                </p>
            </div>
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.takeArchivalSnapshot}`}
                    action={takeSnapshot}
                />
            </div>
        </div>
    );
};
