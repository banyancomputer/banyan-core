import React, { useState } from 'react';
import { useIntl } from 'react-intl';
import { FiChevronRight } from 'react-icons/fi';
import { Bucket } from './Bucket';
import { DatePicker } from '@/components/common/DatePicker';

export interface BucketBillingInfo {
    name: string;
    data: Array<{ memoryAmount: number; serviceName: string; billiedAt: string; cost: string }>;
}

export const BillingHistory = () => {
    const { messages } = useIntl();
    const [isVisible, setIsVisible] = useState(false);
    const [dateRange, setDateRange] = useState({ from: new Date(), to: new Date() });

    const changeDateRange = (startDate: Date, endDate: Date) => {
        setDateRange({ from: startDate, to: endDate });
    };

    const MOCK_DATA: BucketBillingInfo[] = [
        {
            name: 'Bucket test',
            data: [
                {
                    memoryAmount: 6000000000000,
                    serviceName: 'Migration Fees',
                    billiedAt: 'waived (>50 TB)',
                    cost: '0',
                },
                {
                    memoryAmount: 7000000000000,
                    serviceName: 'Egress Fees',
                    billiedAt: '$10/TB/mo',
                    cost: '650',
                },
                {
                    memoryAmount: 6000000000000,
                    serviceName: 'Migration Fees',
                    billiedAt: 'waived (>50 TB)',
                    cost: '0',
                },
                {
                    memoryAmount: 7000000000000,
                    serviceName: 'Egress Fees',
                    billiedAt: '$10/TB/mo',
                    cost: '650',
                },
            ],
        },
        {
            name: 'Bucket test2',
            data: [
                {
                    memoryAmount: 6000000000000,
                    serviceName: 'Migration Fees',
                    billiedAt: 'waived (>50 TB)',
                    cost: '0',
                },
                {
                    memoryAmount: 7000000000000,
                    serviceName: 'Egress Fees',
                    billiedAt: '$10/TB/mo',
                    cost: '650',
                },
            ],
        },
    ];

    return (
        <div className="py-5 px-4 border-1 flex flex-col gap-4 rounded-lg text-text-800 border-border-regular bg-secondaryBackground">
            <div className="flex items-center justify-between ">
                <h3 className="font-semibold mb-1.5">{`${messages.billingHistory}`}</h3>
                <button
                    className={`text-text-600 ${isVisible && 'rotate-90'}`}
                    onClick={() => setIsVisible(prev => !prev)}
                >
                    <FiChevronRight size="20px" />
                </button>
            </div>
            {isVisible &&
                <>
                    <div className="flex items-center justify-between">
                        <p>Metered</p>
                        <DatePicker
                            from={dateRange.from}
                            to={dateRange.to}
                            onChange={changeDateRange}
                        />
                    </div>
                    {
                        MOCK_DATA.map(BucketBillingInfo =>
                            <React.Fragment key={BucketBillingInfo.name}>
                                <Bucket billingInfo={BucketBillingInfo} />
                            </React.Fragment>
                        )
                    }
                </>
            }
        </div>
    );
};
