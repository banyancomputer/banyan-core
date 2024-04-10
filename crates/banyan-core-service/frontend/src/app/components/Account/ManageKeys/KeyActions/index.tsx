import React from 'react';

import { RenameAccessKeyModal } from '@components/common/Modal/RenameAccessKeyModal ';

import { Bucket, BucketKey } from '@app/types/bucket';
import { useModal } from '@contexts/modals';
import { useAppSelector } from '@app/store';
import { AccessKeysClient } from '@/api/accessKeys';
import { useTomb } from '@contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

import { Rename, Trash } from '@static/images/common';

const client = new AccessKeysClient();

export const KeyActions: React.FC<{ bucket: Bucket; bucketKey: BucketKey }> = ({ bucket, bucketKey }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.account.manageKeys.keyActions);
    const { openModal } = useModal();
    const { getBucketsKeys } = useTomb();

    const rename = async () => {
        openModal(<RenameAccessKeyModal bucket={bucket} bucketKey={bucketKey} />);
    };

    const remove = async () => {
        if (bucket.keys.length <= 1) {
            ToastNotifications.error('The final key cannot be disabled or removed without at least one backup.');
            return;
        };
        try {
            await client.deleteAccessKey(bucket.id, bucketKey.id);
            await getBucketsKeys();
        } catch (error: any) {
        };
    };

    return (
        <div className="w-52 text-xs font-medium bg-bucket-actionsBackground rounded-md shadow-md z-10 text-bucket-actionsText overflow-hidden">
            <div
                className="flex items-center gap-2 py-3 px-4 transition-all hover:bg-hover"
                onClick={rename}
            >
                <Rename />
                {messages.rename}
            </div>
            <div
                className="flex items-center gap-2 py-3 px-4 transition-all hover:bg-hover"
                onClick={remove}
            >
                <Trash />
                {messages.removeKey}
            </div>
        </div>
    );
};
