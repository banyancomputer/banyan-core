import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';
import { UploadFileModal } from '@components/common/Modal/UploadFileModal';

import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { stringToBase64 } from '@/app/utils/base64';
import { useAppSelector } from '@/app/store';

export const CreateFolderModal: React.FC<{ bucket: Bucket; onSuccess?: (path: string[]) => void; path: string[], redirect?: boolean }> = ({ bucket, onSuccess = () => { }, path, redirect = false }) => {
    const { closeModal, openModal } = useModal();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.createFolder);
    const { folderAlreadyExists } = useAppSelector(state => state.locales.messages.contexts.tomb);
    const [folderName, setfolderName] = useState('');
    const { createDirectory } = useTomb();
    const navigate = useNavigate();

    const changeName = (event: React.ChangeEvent<HTMLInputElement>) => {
        if (event.target.value.length >= 32) { return; }

        setfolderName(event.target.value);
    };

    const create = async () => {
        try {
            await createDirectory(bucket, path, folderName);
            ToastNotifications.notify(
                messages.folderCreated,
                null,
                messages.viewFolder,
                () => navigate(`/drive/${bucket.id}${path.length ? '?' : ''}${path.map(path => stringToBase64(path)).join('/')}${path.length ? '/' : '?'}${stringToBase64(folderName)}`)
            );
            onSuccess ?
                onSuccess([...path, folderName])
                :
                openModal(<UploadFileModal bucket={bucket} path={[...path, folderName]} />);
        } catch (error: any) {
            if (error.message !== folderAlreadyExists) {
                ToastNotifications.error(`${messages.creationError}`);
            };
        };
    };

    return (
        <div className="w-modal flex flex-col gap-5" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.folderName}`}
                    <input
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-border-darken focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewName}`}
                        value={folderName}
                        onChange={changeName}
                    />
                </label>
            </div>
            <div className="flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.create}`}
                    action={create}
                    disabled={!folderName}
                />
            </div>
        </div >
    );
};
