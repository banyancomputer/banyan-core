import React, { useState } from 'react';
import { useIntl } from 'react-intl';
import { MdDone } from 'react-icons/md';

import { useModal } from '@/contexts/modals';
import { useTomb } from '@/contexts/tomb';
import { ToastNotifications } from '@/utils/toastNotifications';
import { Bucket } from '@/lib/interfaces/bucket';
import { useFolderLocation } from '@/hooks/useFolderLocation';

export const CreateFolderModal: React.FC<{ bucket: Bucket, onSuccess?: () => void }> = ({ bucket, onSuccess = () => { } }) => {
    const { closeModal, openModal } = useModal();
    const { messages } = useIntl();
    const [newName, setNewName] = useState('');
    const { createDirectory } = useTomb();
    const folderLocation = useFolderLocation();

    const create = async () => {
        try {
            await createDirectory(bucket, [...folderLocation, newName]);
            onSuccess();
        } catch (error: any) { };
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
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-lg border-gray-400 focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewBucketName}`}
                        value={newName}
                        onChange={event => setNewName(event.target.value)}
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
                <button
                    className="btn-primary flex-grow py-3 px-4"
                    onClick={create}
                >
                    {`${messages.create}`}
                </button>
            </div>
        </div >
    );
};
