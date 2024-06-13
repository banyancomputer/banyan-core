
import { useEffect } from 'react';

import { KeyManagementTable } from '@components/Account/ManageKeys/KeyManagementTable';
import { Fallback } from '@components/common/Fallback';

import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppDispatch, useAppSelector } from '@store/index';
import { getBucketsKeys } from '@store/tomb/actions';

const ManageKeys = () => {
    const dispatch = useAppDispatch();
    const {buckets, isLoading, tomb,} = useAppSelector(state => state.tomb);

    useEffect(() => {
        if (!tomb) { return; }

        const getKeys = async () => {
            try {
                await dispatch(getBucketsKeys());
            } catch (error: any) {
                ToastNotifications.error('Failed to upload files', 'Try again', getKeys)
            }
        };

        getKeys();
    }, [buckets.length, tomb]);

    return (
        <div className="flex flex-grow flex-col gap-5 p-6">
            <Fallback shouldRender={!isLoading}>
                <KeyManagementTable buckets={buckets} />
            </Fallback>
        </div>
    );
};

export default ManageKeys;
