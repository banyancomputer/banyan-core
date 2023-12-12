import React, { useState } from 'react';
import { useIntl } from 'react-intl';

import { SubmitButton } from '@components/common/SubmitButton';
import { UploadFileModal } from '@components/common/Modal/UploadFileModal';

import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';

export const CreateFolderModal: React.FC<{ bucket: Bucket; onSuccess?: () => void, path: string[] }> = ({ bucket, onSuccess = () => { }, path }) => {
    const { closeModal, openModal } = useModal();
    const { messages } = useIntl();
    const [folderName, setfolderName] = useState('');
    const { createDirectory } = useTomb();

    const changeName = (event: React.ChangeEvent<HTMLInputElement>) => {
        if (event.target.value.length >= 32) { return; }

        setfolderName(event.target.value);
    };

    const create = async () => {
        try {
            await createDirectory(bucket, path, folderName);
            onSuccess();
            openModal(<UploadFileModal bucket={bucket} path={[...path, folderName]} />);
        } catch (error: any) {
            ToastNotifications.error(`${messages.creationError}`, `${messages.tryAgain}`, create);
        };
    };

    return (
        <div className="w-modal flex flex-col gap-5" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.createNewFolder}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.folderName}`}
                    <input
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-lg border-border-darken focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewDriveName}`}
                        value={folderName}
                        onChange={changeName}
                    />
                </label>
            </div>
            <div className="flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <SubmitButton
                    text={`${messages.create}`}
                    action={create}
                    disabled={!folderName}
                />
            </div>
        </div >
    );
};
