import React from 'react';
import { useIntl } from 'react-intl';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { Bucket } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

import { Bolt } from '@static/images/common';

export const TakeSnapshotModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { closeModal } = useModal();
    const { takeColdSnapshot } = useTomb();
    const { messages } = useIntl();

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
            <span className="p-3 w-min rounded-full bg-button-disabled">
                <Bolt width="24px" height="24px" />
            </span>
            <div>
                <h4 className="text-m font-semibold">{`${messages.takeColdSnapshot}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.doYouWantToTakeSnapshot}`}?
                </p>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <SecondaryButton
                    className="w-1/2"
                    action={closeModal}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.takeColdSnapshot}`}
                    action={takeSnapshot}
                    className="w-1/2"
                />
            </div>
        </div>
    );
};
