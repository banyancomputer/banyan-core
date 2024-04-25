import React from 'react';

import { KeyActions } from '@components/Account/ManageKeys/KeyActions';
import { ActionsCell } from '@components/common/ActionsCell';
import { ApproveBucketAccessModal } from '@components/common/Modal/ApproveBucketAccessModal';
import { RemoveBucketAccessModal } from '@components/common/Modal/RemoveBucketAccessModal';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { Bucket, BucketAccess, Bucket as IBucket } from '@/app/types/bucket';
import { UserKeyAccess } from '@/app/types/userKeyAccess'
import { useAppDispatch, useAppSelector } from '@/app/store';
import { openModal } from '@store/modals/slice';
import { ToastNotifications } from '@/app/utils/toastNotifications';

export const KeyManagementTable: React.FC<{ buckets: IBucket[], userKeyAccess: UserKeyAccess[] }> = ({ buckets, userKeyAccess }) => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.account.manageKeys.keyManagementTable);

    const access = JSON.stringify(userKeyAccess);
    console.log('HEY!');
    console.log(access);

    return (
        <div>
            {access}
        </div >
    );
};

