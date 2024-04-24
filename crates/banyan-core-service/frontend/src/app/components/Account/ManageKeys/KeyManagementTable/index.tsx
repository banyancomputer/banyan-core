import React from 'react';

import { KeyActions } from '@components/Account/ManageKeys/KeyActions';
import { ActionsCell } from '@components/common/ActionsCell';
import { ApproveBucketAccessModal } from '@components/common/Modal/ApproveBucketAccessModal';
import { RemoveBucketAccessModal } from '@components/common/Modal/RemoveBucketAccessModal';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { useAppDispatch, useAppSelector } from '@/app/store';
import { Bucket, BucketKey, Bucket as IBucket } from '@/app/types/bucket';
import { openModal } from '@store/modals/slice';
import { ToastNotifications } from '@/app/utils/toastNotifications';

export const KeyManagementTable: React.FC<{ buckets: IBucket[] }> = ({ buckets }) => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.account.manageKeys.keyManagementTable);

    const approveAccess = async (bucket: Bucket, bucketKey: BucketKey) => {
        dispatch(openModal({content: <ApproveBucketAccessModal bucket={bucket} bucketKey={bucketKey} />}));
    };

    const removeAccess = async (bucket: Bucket, bucketKey: BucketKey) => {
        if (bucket.keys.length <= 1) {
            ToastNotifications.error('The final key cannot be disabled or removed without at least one backup.');
            return;
        };
        dispatch(openModal({content: <RemoveBucketAccessModal bucket={bucket} bucketKey={bucketKey} />}));
    };

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
                    {buckets.map(bucket =>
                        <React.Fragment key={bucket.id}>
                            {
                                bucket?.keys?.map(bucketKey =>
                                    <tr key={bucketKey.id} className="border-b-1 border-y-border-regular">
                                        <td className="px-6 py-12 ">
                                            <div className="text-ellipsis overflow-hidden">
                                                {bucketKey.fingerPrint}
                                            </div>
                                        </td>
                                        <td className="px-6 py-12">-</td>
                                        <td className="px-6 py-12">{bucket.name}</td>
                                        <td className="px-6 py-12">-</td>
                                        <td className="px-6 py-12">
                                            <SecondaryButton
                                                text={bucketKey.approved ? messages.disable : messages.enable}
                                                action={() => bucketKey.approved ? removeAccess(bucket, bucketKey) : approveAccess(bucket, bucketKey)}
                                            />
                                        </td>
                                        <td className="px-3 py-12">
                                            {bucket.keys.length >= 1 &&
                                                <ActionsCell
                                                    actions={<KeyActions bucket={bucket} bucketKey={bucketKey} />}
                                                />
                                            }
                                        </td>
                                    </tr>
                                )
                            }
                        </React.Fragment>
                    )}
                </tbody>
            </table >
        </div >
    );
};

