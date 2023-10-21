import React from 'react';
import { useIntl } from 'react-intl';
import { NextPageWithLayout } from '../page';
import SettingsLayout from '@/layouts/SettingsLayout';
import { StorageUsage } from '@/components/common/StorageUsage';
import { ServicesTable } from '@/components/Account/Services/ServicesTable';

export const Services: NextPageWithLayout = () => {
    const { messages } = useIntl();

    return (
        <div className="flex flex-col gap-5 px-4">
            <h2 className="text-lg font-semibold">
                {`${messages.services}`}
            </h2>
            <div className="flex justify-between items-center border-1 rounded-lg text-text-800 border-border-regular">
                <StorageUsage />
            </div>
            <ServicesTable />
        </div>
    );
};

export default Services;

Services.getLayout = (page) => <SettingsLayout>{page}</SettingsLayout>;
