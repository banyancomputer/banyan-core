import React, { useState } from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { useModal } from '@/app/contexts/modals';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useAppSelector } from '@/app/store';

import { Done } from '@static/images/common';

export const RenameFileModal: React.FC<{ bucket: Bucket; file: BrowserObject; path: string[] }> = ({ bucket, file, path }) => {
    const { closeModal } = useModal();
    const { moveTo, getSelectedBucketFiles, selectBucket } = useTomb();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.renameFile);
    const [newName, setNewName] = useState('');
    const folderLocation = useFolderLocation();

    const save = async () => {
        try {
            await moveTo(bucket, [...path, file.name], [...path], newName);
            ToastNotifications.notify(`${messages.fileWasRenamed}`, <Done width="20px" height="20px" />);
            if (path.join('/') === folderLocation.join('/')) {
                await getSelectedBucketFiles(folderLocation);
                closeModal();

                return;
            };
            file.name = newName;
            selectBucket({ ...bucket });
            closeModal();
        } catch (error: any) {
            ToastNotifications.error(`${messages.editError}`, `${messages.tryAgain}`, save);
        };
    };

    return (
        <div className="w-modal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.fileName}`}
                    <input
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-border-darken shadow-sm focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewName}`}
                        value={newName}
                        onChange={event => setNewName(event.target.value)}
                    />
                </label>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.save}`}
                    action={save}
                    disabled={newName.length < 3}
                />
            </div>
        </div >
    );
};
