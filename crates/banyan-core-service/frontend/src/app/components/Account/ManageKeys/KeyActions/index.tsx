import React from 'react';

import { RenameAccessKeyModal } from '@components/common/Modal/RenameAccessKeyModal ';

import { Bucket, BucketKey } from '@app/types/bucket';
import { openModal } from '@store/modals/slice';
import { useAppDispatch, useAppSelector } from '@app/store';
import { AccessKeysClient } from '@/api/accessKeys';
import { useTomb } from '@contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

import { Rename, Trash } from '@static/images/common';

const client = new AccessKeysClient();

export const KeyActions: React.FC<{ bucket: Bucket; bucketKey: BucketKey }> = ({ bucket, bucketKey }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.account.manageKeys.keyActions);
    const { getBucketsKeys } = useTomb();
    const dispatch = useAppDispatch();

    const rename = async () => {
        dispatch(openModal({content: <RenameAccessKeyModal bucket={bucket} bucketKey={bucketKey} />}));
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
        <div className="absolute right-5 w-52 text-xs font-medium bg-bucket-actionsBackground rounded-md shadow-md z-10 text-bucket-actionsText overflow-hidden">
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
