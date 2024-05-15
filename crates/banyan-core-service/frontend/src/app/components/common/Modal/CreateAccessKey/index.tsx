import React, { useState } from 'react';

import { SecondaryButton } from '@components/common/SecondaryButton';
import { PrimaryButton } from '@components/common/PrimaryButton';

import { useTomb } from '@/app/contexts/tomb';
import { useAppDispatch, useAppSelector } from '@/app/store';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { closeModal } from '@/app/store/modals/slice';

export const CreateAccessKey = () => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.createAccessKey);
    const [accessKeyName, setAccessKeyName] = useState('');
    const { createAccessKey, getUserAccessKeys } = useTomb();
    const [pem, setPem] = useState('');

    const changeName = (event: React.ChangeEvent<HTMLInputElement>) => {
        setAccessKeyName(event.target.value);
    };
    const changePem = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
        setPem(event.target.value);
    };

    const create = async () => {
        try {
            await createAccessKey(accessKeyName, pem);
            dispatch(closeModal());
            await getUserAccessKeys();
        } catch (error: any) {
            ToastNotifications.error(messages.creationError);
        };
    };

    const cancel = () => {
        dispatch(closeModal());
    };

    return (
        <div className="w-modal flex flex-col gap-5" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
            </div>
            <label>
                {`${messages.accessKeyName}`}
                <input
                    className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-border-darken focus:outline-none"
                    type="text"
                    placeholder={`${messages.enterKeyName}`}
                    value={accessKeyName}
                    onChange={changeName}
                />
            </label>
            <label>
                {`${messages.pem}`}
                <textarea
                    className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-border-darken focus:outline-none"
                    placeholder={`${messages.enterPem}`}
                    value={pem}
                    onChange={changePem}
                />
            </label>
            <div className="flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={cancel}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.create}`}
                    action={create}
                    disabled={!(accessKeyName && pem)}
                />
            </div>
        </div >
    )
}
