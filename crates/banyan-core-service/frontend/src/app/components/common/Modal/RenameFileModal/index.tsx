import React, { useEffect, useRef, useState } from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { closeModal } from '@store/modals/slice';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useAppDispatch, useAppSelector } from '@/app/store';
import { getSelectedBucketFiles, moveTo } from '@/app/store/tomb/actions';
import { selectBucket } from '@/app/store/tomb/slice';

export const RenameFileModal: React.FC<{ bucket: Bucket; file: BrowserObject; path: string[] }> = ({ bucket, file, path }) => {
    const inputRef = useRef<HTMLInputElement | null>(null);
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.renameFile);
    const [newName, setNewName] = useState(file.name);
    const folderLocation = useFolderLocation();
    const dispatch = useAppDispatch();

    const close = () => {
        dispatch(closeModal());
    };

    const save = async () => {
        try {
            unwrapResult(await dispatch(moveTo({ bucket, from: [...path, file.name], to: [...path], name: newName })));
            ToastNotifications.notify(`${messages.fileWasRenamed}`);
            if (path.join('/') === folderLocation.join('/')) {
                unwrapResult(await dispatch(getSelectedBucketFiles(folderLocation)));
                close();

                return;
            };
            file.name = newName;
            dispatch(selectBucket({ ...bucket }));
            close();
        } catch (error: any) {
            console.error(error)
            ToastNotifications.error(`${messages.editError}`, `${messages.tryAgain}`, save);
        };
    };

    useEffect(() => {
        if (!inputRef.current) return;

        const separatorIndex = file.name.lastIndexOf('.');
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
                    {`${messages.fileName}`}
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
                    disabled={newName === file.name || newName.length < 3}
                />
            </div>
        </div >
    );
};
