import React, { useState } from 'react';
import { useIntl } from 'react-intl';

export const BillingInfo = () => {
    const { messages } = useIntl();
    const [accountName, setAccountName] = useState('BANYAN');
    const [accountHolder, setAccountHolder] = useState('SAM GRONE');

    return (
        <div className="py-5 px-4 border-1 rounded-lg text-text-800 border-border-regular bg-secondaryBackground">
            <div className="mb-4 flex items-center justify-between text-xs font-semibold">
                <h3 className="font-semibold">{`${messages.billingInfo}`}</h3>
                <button>{`${messages.edit}`}</button>
            </div>
            <div className="flex items-center gap-4">
                <div className="flex flex-col flex-grow">
                    <h4 className="mb-4 text-xs font-medium">{`${messages.accountName}`}</h4>
                    <p className="text-text-600 font-semibold">{accountName}</p>
                </div>
                <div className="flex flex-col flex-grow">
                    <h4 className="mb-4 text-xs font-medium">{`${messages.accountHolder}`}</h4>
                    <p className="text-text-600 font-semibold">{accountHolder}</p>
                </div>
            </div>
        </div>
    );
};
