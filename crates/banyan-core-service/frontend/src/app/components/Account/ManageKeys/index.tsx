
import { useEffect } from 'react';

import { KeyManagementTable } from '@components/Account/ManageKeys/KeyManagementTable';
import { Fallback } from '@components/common/Fallback';

import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

const ManageKeys = () => {
    const { buckets, areBucketsLoading, getUserKeyAccess, userKeyAccess, tomb } = useTomb();

    useEffect(() => {
        if (!tomb) { return; }

        const getAccess = async () => {
            try {
                await getUserKeyAccess();
            } catch (error: any) {
                ToastNotifications.error('Failed to get user key access', 'Try again', getAccess)
            }
        };

        getAccess();
    }, [buckets.length, tomb]);

    return (
        <div className="flex flex-grow flex-col gap-5 p-6">
            <Fallback shouldRender={!areBucketsLoading}>
                <KeyManagementTable buckets={buckets} userKeyAccess={userKeyAccess} />
            </Fallback>
        </div>
    );
};

export default ManageKeys;
