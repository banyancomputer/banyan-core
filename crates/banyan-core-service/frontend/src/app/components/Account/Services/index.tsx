import React from 'react';
import { useIntl } from 'react-intl';
import { StorageUsage } from '@/app/components/common/StorageUsage';
import { ServicesTable } from '@/app/components/Account/Services/ServicesTable';

export const Services = () => {
    const { messages } = useIntl();

    return (
        <div className="flex flex-col gap-5 px-10">
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
