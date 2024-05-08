import React from 'react';

import { RenameAccessKeyModal } from '@components/common/Modal/RenameAccessKeyModal';

import { openModal } from '@store/modals/slice';
import { useAppDispatch, useAppSelector } from '@app/store';
import { useTomb } from '@contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

import { Rename, Trash } from '@static/images/common';
import { UserAccessKey } from '@/app/types/userAccessKeys';


export const KeyActions: React.FC<{ accessKey: UserAccessKey }> = ({ accessKey }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.account.manageKeys.keyActions);
    const { } = useTomb();
    const dispatch = useAppDispatch();

    const rename = async () => {
        dispatch(openModal({ content: <RenameAccessKeyModal accessKey={accessKey} /> }));
    };

    const remove = async () => {
        if (accessKey.buckets.length <= 1) {
            ToastNotifications.error('The final key cannot be disabled or removed without at least one backup.');
            return;
        };
        // TODO
        ToastNotifications.error('This functionality is still being implemented.');
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
