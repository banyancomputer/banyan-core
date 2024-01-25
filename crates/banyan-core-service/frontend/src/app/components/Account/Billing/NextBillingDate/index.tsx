import React, { useEffect } from 'react';
import { useIntl } from 'react-intl';

import { useAppDispatch, useAppSelector } from '@/app/store';
import { getSubscriptions } from '@/app/store/billing/actions';
import { useModal } from '@/app/contexts/modals';
import { SubscriptionPlanModal } from '@/app/components/common/Modal/SubscriptionPlanModal';

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

    useEffect(() => {
        dispatch(getSubscriptions());
    }, []);

    return (
        <div className="flex-grow flex flex-col gap-4 p-4 border-1 border-border-regular rounded-lg text-xs">
            <h3 className="text-text-800 text-[18px] font-semibold">Next Billing Date</h3>
            <div className="flex justify-between items-center">
                <div>{`${messages.onDemandStorage}`}</div>
                <div className="text-text-800">{getHotStorageAmount() || 0} TB</div>
            </div>
            {/* <div className="flex justify-between items-center">
                <div className="text-text-800">{`${messages.archivalStorage}`}</div>
                <div className="text-text-800">{selectedSubscription?.features.archival_hard_limit || 0} TB</div>
            </div> */}
            <div className="flex justify-between items-center">
                <div>{`${messages.dataEgress}`}</div>
                <div className="text-text-800">{selectedSubscription?.features?.included_bandwidth} TB</div>
            </div>
            <div className="flex justify-between items-center">
                <div>{`${messages.totalCost}`}</div>
                <div className="text-[20px] font-semibold text-text-900">${selectedSubscription?.pricing?.plan_base || 0}</div>
            </div>
            <button
                onClick={upgragePlan}
                className="w-max px-4 py-2 text-xs font-semibold rounded-md bg-text-200 text-button-primary"
            >
                {`${selectedSubscription?.service_key === 'starter' ? `${messages.upgrade} ${messages.account}` : messages.upgradePlan}`}
            </button>
        </div>
    )
}
