
import { useEffect } from 'react';

import { KeyManagementTable } from '@components/Account/ManageKeys/KeyManagementTable';
import { Fallback } from '@components/common/Fallback';

import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { PrimaryButton } from '../../common/PrimaryButton';
import { PlusBold } from '@/app/static/images/common';
import { useAppDispatch } from '@/app/store';
import { openModal } from '@/app/store/modals/slice';
import { CreateAccessKey } from '../../common/Modal/CreateAccessKey';

const ManageKeys = () => {
    const dispatch = useAppDispatch();
    const { buckets, areAccessKeysLoading, getUserAccessKeys, userAccessKeys, tomb } = useTomb();

    const addKey = () => {
        dispatch(openModal({ content: <CreateAccessKey /> }))
    };

    useEffect(() => {
        if (!tomb) { return; }

        const getAccess = async () => {
            try {
                await getUserAccessKeys();
            } catch (error: any) {
                ToastNotifications.error('Failed to get user access keys', 'Try again', getAccess)
            };
        };

        getAccess();
    }, [buckets.length, tomb]);

    return (
        <div className="flex flex-grow flex-col items-start gap-5 p-6">
            <Fallback shouldRender={!areAccessKeysLoading}>
                <PrimaryButton
                    text="Add Key"
                    icon={<PlusBold />}
                    action={addKey}
                />
                <KeyManagementTable userAccessKeys={userAccessKeys} />
            </Fallback>
        </div>
    );
};

export default ManageKeys;
