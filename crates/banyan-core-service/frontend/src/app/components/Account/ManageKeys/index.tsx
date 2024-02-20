
import { useEffect } from 'react';

import { KeyManagementTable } from '@components/Account/ManageKeys/KeyManagementTable';
import { Fallback } from '@components/common/Fallback';

import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

const ManageKeys = () => {
    const { buckets, areBucketsLoading, tomb, getBucketsKeys } = useTomb();

    useEffect(() => {
        if (!tomb) { return; }

        const getKeys = async () => {
            try {
                await getBucketsKeys();
            } catch (error: any) {
                ToastNotifications.error('Failed to upload files', 'Try again', getKeys)
            }
        };

        getKeys();
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
