import React, { useState } from 'react';
import { useIntl } from 'react-intl';

import { PlusBold } from '@static/images/common';

export const PaymentMethods = () => {
    const { messages } = useIntl();

    const addPaymentMethod = () => {
        /** TODO: implement when stripe will be integrated. */
    };

    return (
        <div className="py-5 px-4 border-1 rounded-lg text-text-800 border-border-regular bg-secondaryBackground">
            <h3 className="font-semibold mb-1.5">{`${messages.paymentMethods}`}</h3>
            <div
                className="w-1/2 flex items-center justify-between gap-4 py-3 px-5 border-1 border-border-regular rounded-xl cursor-pointer"
                onClick={addPaymentMethod}
            >
                <h4 className="text-sm ">{`${messages.addPaymentMethod}`}</h4>
                <span
                    className="flex items-center justify-center w-12 h-12 border-1 border-border-regular rounded-xl text-gray-300"
                >
                    <PlusBold width="30px" height="30px" />
                </span>
            </div>
        </div>
    );
};
