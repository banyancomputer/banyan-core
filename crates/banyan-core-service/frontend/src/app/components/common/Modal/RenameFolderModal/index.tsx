import React, { useEffect, useRef, useState } from 'react';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { closeModal } from '@store/modals/slice';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useAppDispatch, useAppSelector } from '@/app/store';

export const RenameFolderModal: React.FC<{ bucket: Bucket; folder: BrowserObject; path: string[] }> = ({ bucket, folder, path }) => {
    const inputRef = useRef<HTMLInputElement | null>(null);
    const { moveTo, getSelectedBucketFiles, selectBucket } = useTomb();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.renameFolder);
    const [newName, setNewName] = useState(folder.name);
    const folderLocation = useFolderLocation();
    const dispatch = useAppDispatch();

    const close = () => {
        dispatch(closeModal());
    };

    const save = async () => {
        try {
            await moveTo(bucket, [...path, folder.name], [...path], newName);
            ToastNotifications.notify(`${messages.folderWasRenamed}`);
            if (path.join('/') === folderLocation.join('/')) {
                await getSelectedBucketFiles(folderLocation);
                close();

                return;
            };
            folder.name = newName;
            selectBucket({ ...bucket });
            close();
        } catch (error: any) {
            ToastNotifications.error(`${messages.editError}`, `${messages.tryAgain}`, save);
        };
    };

    useEffect(() => {
        if (!inputRef.current) return;

        const separatorIndex = folder.name.lastIndexOf('.');
        inputRef.current.select();
        inputRef.current.selectionEnd = separatorIndex;
    }, [inputRef]);

    return (
        <div className="w-modal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.folderName}`}
                    <input
                        ref={inputRef}
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-border-darken shadow-sm focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewName}`}
                        value={newName}
                        onChange={event => setNewName(event.target.value)}
                    />
                </label>
            </div>
            <div className="mt-3 flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={close}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.save}`}
                    action={save}
                    disabled={newName === folder.name || newName.length < 3}
                />
            </div>
        </div >
    );
};
