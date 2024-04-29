import React from 'react';

import { KeyActions } from '@components/Account/ManageKeys/KeyActions';
import { ActionsCell } from '@components/common/ActionsCell';
import { ApproveBucketAccessModal } from '@components/common/Modal/ApproveBucketAccessModal';
import { RemoveBucketAccessModal } from '@components/common/Modal/RemoveBucketAccessModal';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { UserAccessKey } from '@/app/types/userAccessKeys'
import { useAppDispatch, useAppSelector } from '@/app/store';
import { openModal } from '@store/modals/slice';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { Bucket } from '@/app/types/bucket';
import { AccessKeyRow } from './AccessKeyRow';

export const KeyManagementTable: React.FC<{ userAccessKeys: UserAccessKey[] }> = ({ userAccessKeys }) => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.account.manageKeys.keyManagementTable);

    const approveAccess = async (bucket: Bucket, accessKey: UserAccessKey) => {
        dispatch(openModal({ content: <ApproveBucketAccessModal bucket={bucket} accessKey={accessKey} /> }));
    };

    const removeAccess = async (bucket: Bucket, accessKey: UserAccessKey) => {
        if (accessKey.buckets.length <= 1) {
            ToastNotifications.error('The final key cannot be disabled or removed without at least one backup.');
            return;
        };
        dispatch(openModal({ content: <RemoveBucketAccessModal bucket={bucket} accessKey={accessKey} /> }));
    };

    return (
        <div
            className="max-h-[calc(100vh-300px)] flex-grow overflow-x-auto border-1 border-border-regular"
            id="table"
        >
            <table className="table table-fixed table-pin-rows w-full text-text-600">
                <thead className="border-b-reg text-xxs font-normal text-text-600 border-b-1 border-border-regular">
                    <tr className="border-b-table-cellBackground bg-table-headBackground border-none">
                        <th className="w-32 py-3 px-6 whitespace-break-spaces text-left font-medium">
                            Name
                        </th>
                        <th className="w-1/4 py-3 px-6 text-left font-medium whitespace-pre">
                            Fingerprint
                        </th>
                        <th className="w-1/4 py-3 px-6 text-left font-medium">
                            ID
                        </th>
                        <th className="w-1/4 py-3 px-6 text-left font-medium">
                            User Id
                        </th>
                        <th className="w-1/4 py-3 px-6 text-left font-medium">
                            Created At
                        </th>
                        <th className="w-40"></th>
                    </tr>
                </thead>
                <tbody>
                    {userAccessKeys.map(accessKey =>
                        <AccessKeyRow accessKey={accessKey} key={accessKey.id} />
                    )}
                </tbody>
            </table >
        </div >
    );
};

