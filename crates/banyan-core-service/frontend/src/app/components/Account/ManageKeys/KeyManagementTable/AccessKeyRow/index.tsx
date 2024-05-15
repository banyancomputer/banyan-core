import React, { useState } from 'react';
import { ActionsCell } from '@/app/components/common/ActionsCell'
import { SecondaryButton } from '@/app/components/common/SecondaryButton'
import { useAppDispatch, useAppSelector } from '@/app/store'
import { UserAccessKey } from '@/app/types/userAccessKeys'
import { openModal } from '@/app/store/modals/slice';
import { Bucket } from '@/app/types/bucket';
import { ApproveBucketAccessModal } from '@/app/components/common/Modal/ApproveBucketAccessModal';
import { RemoveBucketAccessModal } from '@/app/components/common/Modal/RemoveBucketAccessModal';
import { KeyActions } from '../../KeyActions';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useTomb } from '@/app/contexts/tomb';

export const AccessKeyRow: React.FC<{ accessKey: UserAccessKey }> = ({ accessKey }) => {
    const { buckets } = useTomb();
    const messages = useAppSelector(state => state.locales.messages.coponents.account.manageKeys.keyManagementTable);
    const dispatch = useAppDispatch();

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
        <tr className="border-b-1 border-y-border-regular">
            <td colSpan={5} className="p-0">
                <table className="access-keys-table table table-fixed table-pin-rows w-full">
                    <thead className="text-text-900">
                        <tr className="bg-mainBackground font-normal border-1 border-border-regular">
                            <th className="w-32 py-3 px-6 text-left font-medium">
                                {accessKey.name}
                            </th>
                            <th className="w-1/4 py-3 px-6 text-left font-medium">
                                <div className="text-ellipsis overflow-hidden">
                                    {accessKey.fingerprint}
                                </div>
                            </th>
                            <th className="w-1/4 py-3 px-6 text-left font-medium">
                                <div className="text-ellipsis overflow-hidden">
                                    {accessKey.id}
                                </div>
                            </th>
                            <th className="w-1/4 py-3 px-6 text-left font-medium">
                                <div className="text-ellipsis overflow-hidden">
                                    {accessKey.userId}
                                </div>
                            </th>
                            <th className="w-1/4 py-3 px-6 text-left font-medium">
                                <div className="text-ellipsis overflow-hidden">
                                    {accessKey.createdAt}
                                </div>
                            </th>

                            <th className="w-40 py-3 px-6 text-left font-medium">
                                <div className="flex items-center justify-end gap-4">
                                    <SecondaryButton
                                        text={accessKey.apiAccess ? messages.disable : messages.enable}
                                    />
                                    <ActionsCell actions={
                                        <KeyActions accessKey={accessKey} />
                                    } />
                                </div>
                            </th>
                        </tr>
                    </thead>
                    {buckets.length ?
                        <tbody>
                            {buckets.map(bucket =>
                                <tr>
                                    <td className="px-6 pl-16 py-3" colSpan={3}>
                                        <div className="text-ellipsis overflow-hidden">
                                            {bucket.name}
                                        </div>
                                        <SecondaryButton
                                            text={
                                                accessKey.buckets.some(b => b.id === bucket.id) ? messages.disable : messages.enable
                                            }
                                            action={() =>
                                                accessKey.buckets.some(b => b.id === bucket.id) ?
                                                    removeAccess(bucket, accessKey) : approveAccess(bucket, accessKey)
                                            }
                                        />
                                    </td>
                                    <td></td>
                                </tr>
                            )}
                        </tbody>
                        :
                        null
                    }
                </table>
            </td>
        </tr>
    )
}
