import React from 'react';
import { Link } from 'react-router-dom';

import { SubscriptionPlanModal } from '../Modal/SubscriptionPlanModal';

import { convertFileSize } from '@/app/utils/storage';
import { useAppDispatch, useAppSelector } from '@store/index';
import { RoutesConfig } from '@/app/routes';
import { openModal } from '@store/modals/slice';

export const StorageUsage = () => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.storageUsage);
    const { selectedSubscription } = useAppSelector(state => state.billing);
    const { storageUsage, storageLimits } = useAppSelector(state => state.tomb);

    const upgragePlan = () => {
        dispatch(openModal({ content: <SubscriptionPlanModal /> }));
    };

    return (
        <div className="w-full flex flex-col gap-2 px-4 py-5 bg-navigation-storageUsageBackground rounded-md ">
            <span className="flex justify-between items-center font-semibold">
                {`${messages.storage}`}
            </span>
            <progress
                className="progress w-full bg-navigation-storageUsageProgressBackground [&::-webkit-progress-value]:bg-navigation-storageUsageProgressValue"
                value={storageUsage.hotStorage * 100}
                max={storageLimits.softLimit / (selectedSubscription?.features.included_hot_replica_count || 2)}
            />
            <span className="text-xs font-medium">
                {` ${messages.used} `}
                <span className="uppercase">
                    {convertFileSize(storageUsage.hotStorage)}
                </span>
                {` ${messages.of} `}
                <span className="uppercase">{convertFileSize(storageLimits.softLimit / selectedSubscription?.features.included_hot_replica_count!)}</span>.
            </span>
            {!selectedSubscription?.pricing &&
                <div className="flex justify-end">
                    <Link
                        onClick={upgragePlan}
                        to={RoutesConfig.Billing.fullPath}
                        className="mr-2 mt-2 text-xs font-semibold text-button-primary cursor-pointer"
                    >
                        {`${messages.upgradePlan}`}
                    </Link>
                </div>
            }
        </div>
    );
};
