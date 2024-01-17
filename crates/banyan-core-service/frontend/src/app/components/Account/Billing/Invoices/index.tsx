import React, { useState } from 'react';
import { useIntl } from 'react-intl';

import { DatePicker } from '@components/common/DatePicker';
import { InvoicesTable } from './InvoicesTable';

import { Check } from '@static/images/account';

/** TODO: rework when api will be ready. */
export class Invoice {
    public date: string = 'Dec 05 , 2023';
    public storageUsed: number = 10000000;
    public description: string = 'On Demand storage';
    public price: string = '$120';
}

export const Invoices = () => {
    const { messages } = useIntl();
    /** TODO: rework when api will be ready. */
    const [billingHistory, setBillingHistory] = useState<Array<any>>([]);
    const [dateRange, setDateRange] = useState({ from: new Date(), to: new Date() });

    const changeDateRange = (startDate: Date, endDate: Date) => {
        setDateRange({ from: startDate, to: endDate });
    };

    return (
        <div className="flex flex-col">
            <div className="mb-4 flex items-center justify-between">
                <h3 className="text-base font-semibold">{`${messages.invoices}`}</h3>
                <DatePicker
                    from={dateRange.from}
                    to={dateRange.to}
                    onChange={changeDateRange}
                />
            </div>
            {billingHistory.length ?
                <InvoicesTable billingHistory={billingHistory} />
                :
                <div className="pt-6 flex flex-col gap-4 items-center">
                    <Check />
                    <h4 className="mt-6 text-base text-text-900 font-medium">{`${messages.noPaymentActivity}`}</h4>
                    <p className="text-sm text-text-600">{`${messages.invoicesWillBeAvailiableHere}`}.</p>
                    <button className="px-4 py-2 text-xs font-semibold rounded-md bg-text-200 text-button-primary">{`${messages.upgrade} ${messages.account}`}</button>
                </div>
            }
        </div>
    )
}
