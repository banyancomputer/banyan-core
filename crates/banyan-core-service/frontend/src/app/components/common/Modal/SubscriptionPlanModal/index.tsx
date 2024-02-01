import React, { useEffect, useState } from 'react'
import { useIntl } from 'react-intl';
import { unwrapResult } from '@reduxjs/toolkit';

import { useAppDispatch, useAppSelector } from '@/app/store'
import { getSubscriptions, subscribe } from '@/app/store/billing/actions';
import { convertSubscriptionsSizes } from '@/app/utils/storage';
import { getHotStorageAmount } from '@/app/utils/subscritions';

export const SubscriptionPlanModal = () => {
    const { messages } = useIntl();
    const dispatch = useAppDispatch()
    const { subscriptions } = useAppSelector(state => state.billing);

    const updateSubscription = async (id: string) => {
        try {
            const redirectUrl = unwrapResult(await dispatch(subscribe(id)));
            window.location.href = redirectUrl.checkout_url;
        } catch (error: any) { };
    };

    useEffect(() => {
        dispatch(getSubscriptions());
    }, []);

    return (
        <div className="w-[1050px] flex flex-col">
            <h4 className="text-lg font-semibold text-center ">{`${messages.choosePlan}`}</h4>
            <p className="mb-8 text-sm text-text-600 text-center">{`${messages.plansDescription}.`}</p>
            <div className="mt-4 grid grid-cols-4">
                <div className="flex flex-col transition-all hover:bg-hover">
                    <div className="h-[300px] px-4 py-2 border-1 border-border-regular"></div>
                    <div className="px-4 py-2 border-1 border-border-regular">{`${messages.hotStorage}`}</div>
                    <div className="px-4 py-2 border-1 border-border-regular">{`${messages.hotReplications}`}</div>
                    <div className="px-4 py-2 border-1 border-border-regular">{`${messages.freeEgress}`}</div>
                </div>
                {subscriptions.map(subscription =>
                    <div className="flex flex-col transition-all hover:bg-hover">
                        <div className="h-[300px] flex flex-col items-start px-4 py-2 border-1 border-border-regular">
                            <h5 className="font-semibold">{subscription.title}</h5>
                            {subscription.service_key === 'starter' ?
                                <span className="mt-3 text-text-600 font-normal text-[11px]">{`${messages.litePlanDescription}.`}</span>
                                :
                                <span className="text-[20px]">
                                    ${subscription?.pricing?.plan_base.toFixed(2)}
                                    <span className='inline-block text-[11px] font-normal leading-5'>/mo</span>
                                </span>
                            }
                            <button
                                className={`mt-11 mb-28 w-full py-3 text-xxs font-semibold leading-4 rounded-lg cursor-pointer ${subscription.service_key === 'starter' ? 'bg-button-disabled text-text-600' : 'bg-button-primary text-button-primaryText'}`}
                                disabled={subscription.currently_active}
                                onClick={() => updateSubscription(subscription.id)}
                            >
                                {subscription.currently_active ? `${messages.currentPlan}` : `${messages.upgradeTo} ${subscription.title}`}
                            </button>
                        </div>
                        <div className="px-4 py-2 border-1 border-border-regular">{convertSubscriptionsSizes(getHotStorageAmount(subscription))}</div>
                        <div className="px-4 py-2 border-1 border-border-regular">{subscription.features.included_hot_replica_count}</div>
                        <div className="px-4 py-2 border-1 border-border-regular">{convertSubscriptionsSizes(subscription?.features?.included_bandwidth!)}</div>
                    </div>
                )}
            </div>
            <p className="mt-4 mb-3 text-xxs font-semibold text-center">{`${messages.needCustomPlan}`}</p>
            <a
                href="mailto:sam@banyan.computer"
                target="_blank"
                className="mx-auto font-bold text-xxs text-button-contactSales"
            >
                {`${messages.contactSales}`}
            </a>
        </div>
    )
}
