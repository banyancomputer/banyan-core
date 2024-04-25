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

    console.log('hey!');
    console.log(JSON.stringify(userKeyAccess));

    return (
        <div
            className="max-h-[calc(100vh-300px)] flex-grow overflow-x-auto border-1 border-border-regular"
            id="table"
        >
            <table className="table table-pin-rows w-full text-text-600">
                <thead className="border-b-reg text-xxs font-normal text-text-600 border-b-1 border-border-regular">
                    <tr className="border-b-table-cellBackground bg-table-headBackground border-none">
                        <th className="py-3 px-6 whitespace-break-spaces text-left font-medium">{messages.key}</th>
                        <th className="py-3 px-6 text-left font-medium whitespace-pre">
                            {messages.device}
                        </th>
                        <th className="py-3 px-6 w-32 text-left font-medium">
                            {messages.drive}
                        </th>
                        <th className="py-3 px-6 w-32 text-left font-medium">
                            {messages.createdOn}
                        </th>
                        <th className="w-16"></th>
                        <th className="w-10"></th>
                    </tr>
                </thead>
                <tbody>

                </tbody>
            </table >
        </div >
    );
};

