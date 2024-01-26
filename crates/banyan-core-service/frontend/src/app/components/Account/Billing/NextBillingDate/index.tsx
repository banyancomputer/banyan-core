import React, { useEffect } from 'react';
import { useIntl } from 'react-intl';

import { useAppDispatch, useAppSelector } from '@/app/store';
import { getSubscriptions, manageSubscriptions } from '@/app/store/billing/actions';
import { useModal } from '@/app/contexts/modals';
import { SubscriptionPlanModal } from '@/app/components/common/Modal/SubscriptionPlanModal';
import { convertSubscriptionsSizes } from '@/app/utils/storage';
import { unwrapResult } from '@reduxjs/toolkit';

export const NextBillingDate = () => {
    const dispatch = useAppDispatch();
    const { selectedSubscription } = useAppSelector(state => state.billing);
    const { messages } = useIntl();
    const { openModal } = useModal();

    const getHotStorageAmount = () => {
        if (!selectedSubscription?.features?.included_hot_storage || !selectedSubscription?.features?.included_hot_replica_count) {
            return 0;
        };

        return selectedSubscription!.features.included_hot_storage / selectedSubscription!.features.included_hot_replica_count;
    };

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
        <div className="flex-grow flex flex-col gap-4 p-4 border-1 border-border-regular rounded-lg text-xs">
            <h3 className="text-text-800 text-[18px] font-semibold">Next Billing Date</h3>
            <div className="flex justify-between items-center">
                <div>{`${messages.onDemandStorage}`}</div>
                <div className="text-text-800">{convertSubscriptionsSizes(getHotStorageAmount() || 10)}</div>
            </div>
            {/* <div className="flex justify-between items-center">
                <div className="text-text-800">{`${messages.archivalStorage}`}</div>
                <div className="text-text-800">{selectedSubscription?.features.archival_hard_limit || 0} TB</div>
            </div> */}
            <div className="flex justify-between items-center">
                <div>{`${messages.dataEgress}`}</div>
                <div className="text-text-800">{convertSubscriptionsSizes(selectedSubscription?.features?.included_bandwidth || 10)}</div>
            </div>
            <div className="flex justify-between items-center">
                <div>{`${messages.totalCost}`}</div>
                <div className="text-[20px] font-semibold text-text-900">${selectedSubscription?.pricing?.plan_base || 0}</div>
            </div>
            {selectedSubscription?.service_key ?
                <button
                    onClick={manage}
                    className="w-max px-4 py-2 text-xs font-semibold rounded-md bg-text-200 text-button-primary"
                >
                    {`${messages.manageSubscriptions}`}
                </button>
                :
                <button
                    onClick={upgragePlan}
                    className="w-max px-4 py-2 text-xs font-semibold rounded-md bg-text-200 text-button-primary"
                >
                    {`${messages.upgrade} ${messages.account}`}
                </button>
            }
        </div>
    )
}
