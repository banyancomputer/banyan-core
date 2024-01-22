
import { useEffect } from 'react';

import { useTomb } from '@/app/contexts/tomb';
import { KeyManagementTable } from '@components/Account/ManageKeys/KeyManagementTable';
import { Fallback } from '@components/common/Fallback';

const ManageKeys = () => {
    const { buckets, areBucketsLoading, tomb, getBucketsKeys } = useTomb();

    useEffect(() => {
        if (!tomb) { return; }

        (async () => {
            try {
                await getBucketsKeys();
            } catch (error: any) { }
        })();
    }, [buckets.length, tomb]);

    return (
        <div className="flex flex-grow flex-col gap-5 px-10 py-6">
            <Fallback shouldRender={!areBucketsLoading}>
                <KeyManagementTable buckets={buckets} />
            </Fallback>
        </div>
    );
};

export default ManageKeys;
