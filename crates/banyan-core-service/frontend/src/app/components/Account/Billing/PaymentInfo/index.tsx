import { CardIcon } from '@static/images/account';
import React from 'react';
import { useIntl } from 'react-intl';

export const PaymentInfo = () => {
    const { messages } = useIntl();

    const edit = () => {
        /** TODO: implement when will be ready. */
    };

    return (
        <div className="flex-grow flex flex-col gap-4 p-4 border-1 border-border-regular rounded-lg text-xs text-text-800">
            <div className="flex justify-between items-center text-[18px] font-semibold">
                <div>
                    {`${messages.paymentInfo}`}
                </div>
                <div className="text-[14px] cursor-pointer">{`${messages.edit}`}</div>
            </div>
            <div className="flex flex-col gap-1.5">
                <div>
                    {`${messages.accountHolder}`}
                </div>
                <div className="font-semibold">
                    N/A
                </div>
            </div>
            <div className="flex flex-col gap-1.5">
                <div>
                    {`${messages.billingAddress}`}
                </div>
                <div className="font-semibold">
                    N/A
                </div>
            </div>
            <div className="flex items-start gap-10">
                <div className="flex flex-col gap-2">
                    <div>Card number</div>
                    <div className="flex items-center gap-2 text-text-900 font-medium"><CardIcon /> N/A</div>
                </div>
                <div className="flex flex-col gap-2">
                    <div>Expiry Date</div>
                    <div className="text-text-900 font-medium">-</div>
                </div>
            </div>
        </div>
    )
}
