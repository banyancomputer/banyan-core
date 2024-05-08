import React from 'react';

import { RenameAccessKeyModal } from '@components/common/Modal/RenameAccessKeyModal';

import { Bucket } from '@app/types/bucket';
import { openModal } from '@store/modals/slice';
import { useAppDispatch, useAppSelector } from '@app/store';
import { useTomb } from '@contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

import { Rename, Trash } from '@static/images/common';
import { UserAccessKey } from '@/app/types/userAccessKeys';

export const BucketKeyActions: React.FC<{ bucket: Bucket; accessKey: UserAccessKey }> = ({ bucket, accessKey }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.account.manageKeys.keyActions);
    const { } = useTomb();
    const dispatch = useAppDispatch();

    const rename = async () => {
        // dispatch(openModal({ content: <RenameAccessKeyModal bucket={bucket} bucketAccess={bucketAccess} /> }));
    };

    const remove = async () => {
        if (accessKey.buckets.length <= 1) {
            ToastNotifications.error('The final key cannot be disabled or removed without at least one backup.');
            return;
        };
        try {
            ToastNotifications.error('This functionality is still being implemented.');
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
