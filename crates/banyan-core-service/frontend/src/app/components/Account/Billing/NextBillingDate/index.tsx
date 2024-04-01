import { useEffect } from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { SubscriptionPlanModal } from '@/app/components/common/Modal/SubscriptionPlanModal';

import { useAppDispatch, useAppSelector } from '@/app/store';
import { getSubscriptions, manageSubscriptions } from '@/app/store/billing/actions';
import { useModal } from '@/app/contexts/modals';
import { convertFileSize, convertSubscriptionsSizes } from '@/app/utils/storage';
import { getHotStorageAmount } from '@/app/utils/subscritions';
import { getDateLabel } from '@/app/utils/date';
import { useTomb } from '@app/contexts/tomb';

export const NextBillingDate = () => {
    const dispatch = useAppDispatch();
    const { selectedSubscription } = useAppSelector(state => state.billing);
    const { subscriptionValidUntil, monthlyEggress } = useAppSelector(state => state.session.user);
    const messages = useAppSelector(state => state.locales.messages.coponents.account.billing.invoices.nextBillingDate);
    const { openModal } = useModal();
    const { storageUsage, storageLimits } = useTomb();

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
            <div className="flex justify-between items-end">
                <div className="flex flex-col gap-2">
                    <h3 className="text-text-900 text-[18px] font-semibold">{selectedSubscription?.title}</h3>
                    <h3 className="text-text-900 text-xs font-normal">Next Billing Date</h3>
                </div>
                <span>{subscriptionValidUntil ? getDateLabel(new Date(subscriptionValidUntil).getTime() / 1000) : '-'}</span>
            </div>
            <div className="flex justify-between items-center">
                <div className="font-medium">{messages.totalCost}</div>
                <div className="font-semibold text-text-900">${selectedSubscription?.pricing?.plan_base.toFixed(2) || 0}</div>
            </div>
            <div className="flex flex-col gap-2">
                <div className="flex justify-between items-center">
                    <div>{messages.onDemandStorage}</div>
                    <div className="text-text-800 font-medium">{
                        `${convertFileSize(storageUsage.hotStorage)} of ${convertSubscriptionsSizes(getHotStorageAmount(selectedSubscription))}`
                    }</div>
                </div>
                <progress
                    className="progress w-full [&::-webkit-progress-value]:bg-button-primary"
                    value={storageUsage.hotStorage}
                    max={storageLimits.softLimit / (selectedSubscription?.features.included_hot_replica_count || 2)}
                />
            </div>
            <div className="flex flex-col gap-2">
                <div className="flex justify-between items-center">
                    <div className="text-text-800">{messages.archivalStorage}</div>
                    <div className="text-text-800">{`${convertFileSize(storageUsage.archivalStorage)} of ${convertSubscriptionsSizes(getHotStorageAmount(selectedSubscription))}`}</div>
                </div>
                <progress
                    className="progress w-full [&::-webkit-progress-value]:bg-button-primary"
                    value={storageUsage.archivalStorage}
                    max={storageLimits.softLimit / (selectedSubscription?.features.included_hot_replica_count || 2)}
                />
            </div>
            <div className="flex flex-col gap-2">
                <div className="flex justify-between items-center">
                    <div>{messages.dataEggress}</div>
                    <div className="text-text-800 font-medium">{
                        `${convertFileSize(monthlyEggress)} of ${convertSubscriptionsSizes(selectedSubscription?.features?.included_bandwidth!)}`}</div>
                </div>
                <progress
                    className="progress w-full  [&::-webkit-progress-value]:bg-[#57221E]"
                    value={monthlyEggress}
                    max={selectedSubscription?.features?.included_bandwidth!}
                />
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
