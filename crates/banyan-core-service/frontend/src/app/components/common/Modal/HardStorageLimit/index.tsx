import React from 'react';
import { useNavigate } from 'react-router-dom';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';
import { SubscriptionPlanModal } from '@components/common/Modal/SubscriptionPlanModal';

import { closeModal, openModal } from '@store/modals/slice';
import { useAppDispatch, useAppSelector } from '@store/index';
import { RoutesConfig } from '@/app/routes';

import { OutOfStorageIcon } from '@/app/static/images/common/modal';

export const HardStorageLimit = () => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.hardStorageLimit);
    const navigate = useNavigate();
    const { selectedSubscription } = useAppSelector(state => state.billing);

    const close = () => {
        dispatch(closeModal());
    };

    const upgradePlan = () => {
        if (selectedSubscription?.pricing) {
            navigate(RoutesConfig.Billing.fullPath);
            close();
            return;
        };

        dispatch(openModal({ content: <SubscriptionPlanModal /> }));
    };

    return (
        <div className="w-[530px] flex flex-col rounded">
            <div className="flex justify-center items-center py-10 bg-navigation-secondary">
                <OutOfStorageIcon />
            </div>
            <div className="flex flex-col gap-6 p-6">
                <div>
                    <h5 className="mb-2 text-base font-semibold">{`${messages.title}`}</h5>
                    <p className="text-xs">{`${messages.subtitle}`}</p>
                </div>
                <div className="ml-auto mt-3 w-1/2 flex items-center gap-3 text-xs" >
                    <SecondaryButton
                        action={close}
                        text={`${messages.cancel}`}
                    />
                    <PrimaryButton
                        action={upgradePlan}
                        text={`${messages.upgradePlan}`}
                    />
                </div>
            </div>
        </div>
    )
}
