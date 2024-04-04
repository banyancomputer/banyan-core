import React from 'react';

import { KeyActions } from '@components/Account/ManageKeys/KeyActions';
import { ActionsCell } from '@components/common/ActionsCell';
import { ApproveBucketAccessModal } from '@components/common/Modal/ApproveBucketAccessModal';
import { RemoveBucketAccessModal } from '@components/common/Modal/RemoveBucketAccessModal';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { useAppSelector } from '@/app/store';
import { Bucket, BucketAccess, Bucket as IBucket } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';
import { ToastNotifications } from '@/app/utils/toastNotifications';

export const KeyManagementTable: React.FC<{ buckets: IBucket[] }> = ({ buckets }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.account.manageKeys.keyManagementTable);
    const { openModal } = useModal();

    const approveAccess = async (bucket: Bucket, bucketAccess: BucketAccess) => {
        openModal(<ApproveBucketAccessModal bucket={bucket} bucketAccess={bucketAccess} />);
    };

    const removeAccess = async (bucket: Bucket, bucketAccess: BucketAccess) => {
        if (bucket.access.length <= 1) {
            ToastNotifications.error('The final key cannot be disabled or removed without at least one backup.');
            return;
        };
        openModal(<RemoveBucketAccessModal bucket={bucket} bucketAccess={bucketAccess} />);
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
                                bucket?.access?.map(bucketAccess =>
                                    <tr key={bucketAccess.user_key_id} className="border-b-1 border-y-border-regular">
                                        <td className="px-6 py-12 ">
                                            <div className="text-ellipsis overflow-hidden">
                                                {bucketAccess.fingerprint}
                                            </div>
                                        </td>
                                        <td className="px-6 py-12">-</td>
                                        <td className="px-6 py-12">{bucket.name}</td>
                                        <td className="px-6 py-12">-</td>
                                        <td className="px-6 py-12">
                                            <SecondaryButton
                                                // TODO make this language compatible
                                                text={bucketAccess.state}
                                                action={() => bucketAccess.state == "approved" ? removeAccess(bucket, bucketAccess) : approveAccess(bucket, bucketAccess)}
                                            />
                                        </td>
                                        <td className="px-3 py-12">
                                            {bucket.access.length >= 1 &&
                                                <ActionsCell
                                                    actions={<KeyActions bucket={bucket} bucketAccess={bucketAccess} />}
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

