import React, { useEffect, useState } from 'react'
import { useIntl } from 'react-intl';
import { unwrapResult } from '@reduxjs/toolkit';

import { SubmitButton } from '../../SubmitButton';

import { useAppDispatch, useAppSelector } from '@/app/store'
import { useModal } from '@/app/contexts/modals';
import { getSubscriptions, subscribe } from '@/app/store/billing/actions';
import { Select } from '../../Select';

export const SubscriptionPlanModal = () => {
    const { messages } = useIntl();
    const dispatch = useAppDispatch()
    const { closeModal } = useModal();
    const { subscriptions, selectedSubscription } = useAppSelector(state => state.billing);
    const [selectedPlanId, setSelectedPlanId] = useState('');

    const selectSubscription = (subscription: string) => {
        setSelectedPlanId(subscription);
    };

    const updateSubscription = async () => {
        try {
            const redirectUrl = unwrapResult(await dispatch(subscribe(selectedPlanId)));
            window.location.href = redirectUrl.checkout_url;
        } catch (error: any) { };
    };

    useEffect(() => {
        dispatch(getSubscriptions());
    }, []);

    return (
        <div className="w-modal flex flex-col gap-8">
            <h4 className="text-m font-semibold ">{`${messages.upgradePlan}`}</h4>
            <Select
                selectedOption={selectedPlanId}
                placeholder='Starter Plan'
                onChange={selectSubscription}
                options={subscriptions.map(subscription => ({ label: subscription.title, value: subscription.id }))}
            />
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.close}`}
                </button>
                <SubmitButton
                    disabled={selectedSubscription?.id === selectedPlanId}
                    text={`${messages.upgrade}`}
                    action={updateSubscription}
                />
            </div>
        </div>
    )
}
