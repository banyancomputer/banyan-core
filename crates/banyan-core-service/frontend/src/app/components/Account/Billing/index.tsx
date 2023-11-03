import React from 'react';
import { useIntl } from 'react-intl';

import { ErrorBanner } from '@components/Account/Billing/ErrorBanner';
import { BillingInfo } from '@/app/components/Account/Billing/BillingInfo';
import { PaymentMethods } from '@/app/components/Account/Billing/PaymentMethods';
import { BillingHistory } from '@/app/components/Account/Billing/BillingHistory';

export const Billing = () => {
    const { messages } = useIntl();

    return (
        <div className="flex flex-col gap-5 px-4">
            <h2 className="text-lg font-semibold">
                {`${messages.billing}`}
            </h2>
            <ErrorBanner title={`${messages.paymentIssue}`} description={`${messages.updatePaymentMethod}`} />
            <BillingInfo />
            <PaymentMethods />
            <BillingHistory />
        </div>
    );
};

export default Billing;
