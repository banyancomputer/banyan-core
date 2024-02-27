import React from 'react';

import { KeyActions } from '@components/Account/ManageKeys/KeyActions';
import { ActionsCell } from '@components/common/ActionsCell';

import { useAppSelector } from '@/app/store';
import { Bucket as IBucket } from '@/app/types/bucket';

export const KeyManagementTable: React.FC<{ buckets: IBucket[] }> = ({ buckets }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.account.manageKeys.keyManagementTable);

    return (
        <div
            className="max-h-[calc(100vh-300px)] overflow-x-auto border-2 border-border-regular bg-secondaryBackground rounded-xl"
            id="table"
        >
            <table className="table table-pin-rows key-management-table w-full text-text-600 rounded-xl">
                <thead className="border-b-reg text-xxs font-normal text-text-600 border-b-2 border-border-regular">
                    <tr className="border-b-table-cellBackground bg-table-headBackground border-none">
                        <th className="py-3 px-6 w-44 whitespace-break-spaces text-left font-medium">{messages.device}</th>
                        <th className="py-3 px-6 text-left font-medium whitespace-pre">
                            {messages.client}
                        </th>
                        <th className="py-3 px-6 text-left font-medium">
                            {messages.fingerprint}
                        </th>
                        <th className="py-3 px-6 w-32 text-left font-medium">
                            {messages.status}
                        </th>
                        <th className="w-16"></th>
                    </tr>
                </thead>
                <tbody>
                    {buckets.map(bucket =>
                        <React.Fragment key={bucket.id}>
                            {
                                bucket?.keys?.map(bucketKey =>
                                    <tr key={bucketKey.id} className="border-b-2 border-y-border-regular">
                                        <td className="px-3 py-12">{bucket.name}</td>
                                        <td className="px-3 py-12">{bucketKey.id}</td>
                                        <td className="px-3 py-12">{bucketKey.fingerPrint}</td>
                                        <td className="px-3 py-12">{bucketKey.approved ? messages.approved : messages.noAccess}</td>
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
        </div>
    );
};

