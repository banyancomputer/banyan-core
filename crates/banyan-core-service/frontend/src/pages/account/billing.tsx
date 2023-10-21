import React from 'react';
import { useIntl } from 'react-intl';

import { NextPageWithLayout } from '../page';
import SettingsLayout from '@/layouts/SettingsLayout';
import { ErrorBanner } from "@components/Account/Billing/ErrorBanner"
import { BillingInfo } from '@/components/Account/Billing/BillingInfo';
import { PaymentMethods } from '@/components/Account/Billing/PaymentMethods';
import { BillingHistory } from '@/components/Account/Billing/BillingHistory';

export const Billing: NextPageWithLayout = () => {
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

Billing.getLayout = (page) => <SettingsLayout>{page}</SettingsLayout>;
