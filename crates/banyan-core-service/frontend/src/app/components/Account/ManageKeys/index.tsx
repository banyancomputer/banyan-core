
import { useIntl } from 'react-intl';
import { useEffect } from 'react';

import { useTomb } from '@/app/contexts/tomb';
import { KeyManagementTable } from '@/app/components/Account/ManageKeys/KeyManagementTable';
import { Fallback } from '@/app/components/common/Fallback';

const ManageKeys = () => {
    const { buckets, areBucketsLoading, tomb, getBucketsKeys } = useTomb();
    const { messages } = useIntl();

    useEffect(() => {
        if (!tomb) { return; }

        (async () => {
            await getBucketsKeys();
        })();
    }, [buckets.length, tomb]);

    return (
        <div className="flex flex-grow flex-col gap-5 px-10">
            <h2 className="text-lg font-semibold">
                {`${messages.manageKeyAccess}`}
            </h2>
            <Fallback shouldRender={!areBucketsLoading}>
                <KeyManagementTable buckets={buckets} />
            </Fallback>
        </div>
    );
};

export default ManageKeys;
