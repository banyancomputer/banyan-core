import React, { useEffect, useState } from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { DatePicker } from '@components/common/DatePicker';
import { InvoicesTable } from './InvoicesTable';
import { SubscriptionPlanModal } from '@/app/components/common/Modal/SubscriptionPlanModal';
import { openModal } from '@store/modals/slice';

import { useAppDispatch, useAppSelector } from '@/app/store';
import { getInvoices, manageSubscriptions } from '@/app/store/billing/actions';

import { Check } from '@static/images/account';

export const Invoices = () => {
    const dispatch = useAppDispatch();
    const { invoices, selectedSubscription } = useAppSelector(state => state.billing);
    const messages = useAppSelector(state => state.locales.messages.coponents.account.billing.invoices);
    const [dateRange, setDateRange] = useState({ from: new Date(), to: new Date() });

    const changeDateRange = (startDate: Date, endDate: Date) => {
        setDateRange({ from: startDate, to: endDate });
    };

    const upgragePlan = () => {
        dispatch(openModal({content: <SubscriptionPlanModal />}));
    };

    const manage = async () => {
        try {
            const { portal_url } = unwrapResult(await dispatch(manageSubscriptions()));
            window.location.href = portal_url;
        } catch (error: any) { }
    };

    useEffect(() => {
        dispatch(getInvoices());
    }, [dateRange]);

    return (
        <div className="flex flex-col">
            <div className="mb-4 flex items-center justify-between">
                <h3 className="text-base font-semibold">{`${messages.title}`}</h3>
                {/* <DatePicker
                    from={dateRange.from}
                    to={dateRange.to}
                    onChange={changeDateRange}
                /> */}
            </div>
            {invoices.length ?
                <InvoicesTable invoices={invoices} />
                :
                <>
                    <div className="pt-6 flex flex-col gap-4 items-center">
                        <Check />
                        <h4 className="mt-6 text-base text-text-900 font-medium">{messages.emptyStateTitle}</h4>
                        <p className="text-sm text-text-600">{messages.emptyStateDescription}.</p>
                        {selectedSubscription?.service_key === 'starter' ?
                            <button
                                className="btn-secondary px-4 py-2 text-xs font-semibold rounded-md"
                                onClick={upgragePlan}
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
                </>
            }
        </div>
    )
}
