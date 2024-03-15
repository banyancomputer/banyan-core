import React, { useEffect } from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { SubscriptionPlanModal } from '@/app/components/common/Modal/SubscriptionPlanModal';

import { useAppDispatch, useAppSelector } from '@/app/store';
import { getSubscriptions, manageSubscriptions } from '@/app/store/billing/actions';
import { useModal } from '@/app/contexts/modals';
import { convertSubscriptionsSizes } from '@/app/utils/storage';
import { getHotStorageAmount } from '@/app/utils/subscritions';
import { getDateLabel } from '@/app/utils/date';

export const NextBillingDate = () => {
    const dispatch = useAppDispatch();
    const { selectedSubscription } = useAppSelector(state => state.billing);
    const { subscriptionValidUntil } = useAppSelector(state => state.session.user);
    const messages = useAppSelector(state => state.locales.messages.coponents.account.billing.invoices.nextBillingDate);
    const { openModal } = useModal();


    const upgragePlan = () => {
        openModal(<SubscriptionPlanModal />);
    };

    const manage = async () => {
        try {
            const { portal_url } = unwrapResult(await dispatch(manageSubscriptions()));
            window.location.href = portal_url;
        } catch (error: any) { }
    };

    useEffect(() => {
        dispatch(getSubscriptions());
    }, []);

    return (
        <div className="flex-grow flex flex-col gap-4 p-4 border-1 border-border-regular bg-secondaryBackground rounded-lg text-xs">
            <div className="flex justify-between items-center">
                <h3 className="text-text-800 text-[18px] font-semibold">Next Billing Date</h3>
                <span>{subscriptionValidUntil ? getDateLabel(new Date(subscriptionValidUntil).getTime() / 1000) : '-'}</span>
            </div>
            <div className="flex justify-between items-center">
                <div>{messages.onDemandStorage}</div>
                <div className="text-text-800">{convertSubscriptionsSizes(getHotStorageAmount(selectedSubscription))}</div>
            </div>
            {/* <div className="flex justify-between items-center">
                <div className="text-text-800">{messages.archivalStorage}</div>
                <div className="text-text-800">{selectedSubscription?.features.archival_hard_limit || 0} TB</div>
            </div> */}
            <div className="flex justify-between items-center">
                <div>{messages.dataEggress}</div>
                <div className="text-text-800">{convertSubscriptionsSizes(selectedSubscription?.features?.included_bandwidth!)}</div>
            </div>
            <div className="flex justify-between items-center">
                <div>{messages.totalCost}</div>
                <div className="text-[20px] font-semibold text-text-900">${selectedSubscription?.pricing?.plan_base.toFixed(2) || 0}</div>
            </div>
            {selectedSubscription?.service_key === 'starter' ?
                <button
                    onClick={upgragePlan}
                    className="btn-secondary w-max px-4 py-2 text-xs font-semibold rounded-md"
                >
                    {messages.upgradeAccount}
                </button>
                :
                <button
                    onClick={manage}
                    className="btn-secondary w-max px-4 py-2 text-xs font-semibold rounded-md"
                >
                    {messages.manageSubscriptions}
                </button>
            }
        </div>
    )
}
