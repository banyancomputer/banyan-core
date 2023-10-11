import React, { useState } from 'react';
import { useIntl } from 'react-intl';
import { MdDone } from 'react-icons/md';

import { useModal } from '@/contexts/modals';
import { Bucket, BucketFile } from '@/lib/interfaces/bucket';
import { useTomb } from '@/contexts/tomb';
import { ToastNotifications } from '@/utils/toastNotifications';
import { useFolderLocation } from '@/hooks/useFolderLocation';

export const RenameFileModal: React.FC<{ bucket: Bucket; file: BucketFile }> = ({ bucket, file }) => {
    const { closeModal } = useModal();
    const { moveTo, getSelectedBucketFiles } = useTomb();
    const { messages } = useIntl();
    const [newName, setNewName] = useState('');
    const folderLocation = useFolderLocation();

    const save = async () => {
        try {
            await moveTo(bucket, [...folderLocation, file.name], [...folderLocation, newName]);
            ToastNotifications.notify(`${messages.fileWasRenamed}`, <MdDone size="20px" />);
            await getSelectedBucketFiles(folderLocation);
            closeModal();
        } catch (error: any) {
            ToastNotifications.error(`${messages.editError}`, `${messages.tryAgain}`, save);
        }
    };

    return (
        <div className="w-modal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.renameFile}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.fileName}`}
                    <input
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-lg border-inputBorder shadow-sm focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewBucketName}`}
                        value={newName}
                        onChange={event => setNewName(event.target.value)}
                    />
                </label>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary flex-grow py-3 px-4"
                    onClick={save}
                >{`${messages.save}`}</button>
            </div>
        </div >
    );
};
