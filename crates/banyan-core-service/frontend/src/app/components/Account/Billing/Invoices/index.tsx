import React, { useState } from 'react';
import { useIntl } from 'react-intl';

import { DatePicker } from '@components/common/DatePicker';

import { Check } from '@static/images/account';

export const Invoices = () => {
    const { messages } = useIntl();
    /** TODO: rework when epi will be ready. */
    const [billingInfo, setBillingInfo] = useState<Array<any>>([]);
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
            <div className="pt-6 flex flex-col gap-4 items-center">
                <Check />
                <h4 className="mt-6 text-base text-text-900 font-medium">{`${messages.noPaymentActivity}`}</h4>
                <p className="text-sm text-text-600">{`${messages.invoicesWillBeAvailiableHere}`}.</p>
                <button className="px-4 py-2 text-xs font-semibold rounded-md bg-text-200 text-button-primary">{`${messages.upgrade} ${messages.account}`}</button>
            </div>
        </div>
    )
}
