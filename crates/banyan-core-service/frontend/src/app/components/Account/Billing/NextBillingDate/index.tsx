import React from 'react';
import { useIntl } from 'react-intl';

export const NextBillingDate = () => {
    const { messages } = useIntl();
    return (
        <div className="flex-grow flex flex-col gap-4 p-4 border-1 border-border-regular rounded-lg text-xs">
            <h3 className="text-text-800 text-[18px] font-semibold">Next Billing Date</h3>
            <div className="flex justify-between items-center">
                <div>{`${messages.onDemandStorage}`}</div>
                <div className="text-text-800">12 TB</div>
            </div>
            <div className="flex justify-between items-center">
                <div className="text-text-800">{`${messages.archivalStorage}`}</div>
                <div className="text-text-800">0 TB</div>
            </div>
            <div className="flex justify-between items-center">
                <div>{`${messages.dataEgress}`}</div>
                <div className="text-text-800">0 TB</div>
            </div>
            <div className="flex justify-between items-center">
                <div>{`${messages.totalCost}`}</div>
                <div className="text-[20px] font-semibold text-text-900">$0</div>
            </div>
            <button className="w-max px-4 py-2 text-xs font-semibold rounded-md bg-text-200 text-button-primary">{`${messages.upgrade} ${messages.account}`}</button>
        </div>
    )
}
