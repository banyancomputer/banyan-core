import React, { useEffect, useMemo, useState } from 'react';

import { Select } from '@components/common/Select';
import { AddNewOption } from '@components/common/Select/AddNewOption';
import { CreateDriveModal } from '@components/common/Modal/CreateDriveModal';
import { FolderSelect } from '@components/common/FolderSelect';
import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { closeModal, openModal } from '@store/modals/slice';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useFilesUpload } from '@/app/contexts/filesUpload';
import { useAppDispatch, useAppSelector } from '@/app/store';

import { Upload } from '@static/images/buckets';

export const UploadFileModal: React.FC<{
    bucket?: Bucket | null;
    folder?: BrowserObject;
    path: string[];
    bucketId?: string;
    createdFolderPath?: string[];
    driveSelect?: boolean;
}> = ({ bucket, folder, path, bucketId, createdFolderPath, driveSelect = false }) => {
    const dispatch = useAppDispatch();
    const { uploadFiles } = useFilesUpload();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.uploadFile);
    const { buckets } = useAppSelector(state => state.tomb);
    const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(bucket || null);
    const [selectedFolder, setSelectedFolder] = useState<string[]>(path);
    const [previewFiles, setPreviewFiles] = useState<FileList | null>(null);
    const [fileSizeError, setFileSizeError] = useState(false);
    const isUploadDataFilled = useMemo(() => Boolean(selectedBucket && previewFiles?.length), [selectedBucket, previewFiles]);

    const selectBucket = (bucket: Bucket) => {
        setSelectedBucket(bucket);
    };

    const selectFolder = (option: string[]) => {
        setSelectedFolder(option);
    };

    const handleChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
        if (!event.target.files) { return; };

        if (Array.from(event.target.files).some(file => file.size >= 1e+8)) {
            setFileSizeError(true);

            return;
        };
        setPreviewFiles(event.target.files);
    };

    const handleDrop = async (event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();

        if (!event.dataTransfer.files) { return; };

        if (Array.from(event.dataTransfer.files).some(file => file.size >= 1e+8)) {
            setFileSizeError(true);

            return;
        };

        setPreviewFiles(event.dataTransfer.files);
    };

    const handleDrag = async (event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();
    };

    const close = () => {
        dispatch(closeModal());
    };

    const upload = async () => {
        if (!previewFiles) { return; };

        try {
            close();
            await uploadFiles(previewFiles!, selectedBucket!, selectedFolder.length ? selectedFolder : [], folder);
        } catch (error: any) {
            close();
            ToastNotifications.error(`${messages.uploadError}`, `${messages.tryAgain}`, upload);
        };
    };

    const returnToUploadModal = (id: string) => {
        dispatch(openModal({
            content: <UploadFileModal path={path} bucketId={id} driveSelect={driveSelect} />,
            path
        }));
    };

    const addNewBucket = () => {
        dispatch(openModal({
            content: <CreateDriveModal onSuccess={returnToUploadModal} />,
            onBack: () => {
                dispatch(openModal({
                    content: <UploadFileModal path={path} bucketId={bucketId} driveSelect={driveSelect} />,
                    path
                }))
            }
        }));
    };

    useEffect(() => {
        return () => {
            setPreviewFiles(null);
        };
    }, []);

    useEffect(() => {
        if (!bucketId) return;

        setSelectedBucket(buckets.find(bucket => bucket.id === bucketId) || null);
    }, [bucketId]);

    useEffect(() => {
        if (!createdFolderPath) return;

        setSelectedFolder(createdFolderPath);
    }, [createdFolderPath]);

    return (
        <div className="w-[530px] flex flex-col gap-4">
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
            </div>
            {
                driveSelect ?
                    <div>
                        <span className="inline-block mb-1 text-xs font-normal">{`${messages.selectDrive}`}:</span>
                        <Select
                            selectedOption={selectedBucket}
                            onChange={selectBucket}
                            options={buckets.filter(bucket => bucket.bucketType !== 'backup' && !bucket.locked).map(bucket => ({ value: bucket, label: bucket.name }))}
                            placeholder={`${messages.selectDrive}`}
                            initialOption={<AddNewOption label={`${messages.createNewDrive}`} action={addNewBucket} />}
                        />
                    </div>
                    :
                    null
            }
            {(selectedBucket || bucket) &&
                <div>
                    <span className="inline-block mb-1 text-xs font-normal">{`${messages.selectFolder}`}:</span>
                    <FolderSelect
                        selectedFolder={selectedFolder}
                        onChange={selectFolder}
                        selectedBucket={selectedBucket!}
                        onFolderCreation={
                            (createdFolderPath?: string[]) =>
                                dispatch(openModal({
                                    content: <UploadFileModal
                                        path={path}
                                        bucket={selectedBucket}
                                        bucketId={bucketId}
                                        folder={folder}
                                        createdFolderPath={createdFolderPath}
                                        driveSelect={driveSelect}
                                    />,
                                    path
                                }))
                        }
                    />
                </div>
            }
            <label
                className="mt-2 flex flex-col items-center justify-center gap-4 px-6 py-4 pt-7 border-2 border-dashed border-border-darken  text-xs cursor-pointer"
                onDrop={handleDrop}
                onDragOver={handleDrag}
            >
                {previewFiles ?
                    <React.Fragment >
                        {Array.from(previewFiles).map(file =>
                            <span
                                className="w-full overflow-hidden text-ellipsis whitespace-nowrap"
                                key={file.name}
                            >
                                {file.name}
                            </span>
                        )}
                    </React.Fragment>
                    :
                    <>
                        <Upload />
                        <span className="flex flex-col gap-2 items-center text-text-900 font-light">
                            <b className="">{`${messages.clickToUpload}`} </b>
                            <b className="text-text-600">{`${messages.maxFileSize}`} </b>
                        </span>
                    </>
                }
                <input
                    type="file"
                    multiple
                    className="hidden"
                    onChange={handleChange}
                />
            </label>

            {fileSizeError &&
                <div className="flex justify-center text-error ">
                    {messages.maxFileSizeError}
                    <a
                        className='ml-1 underline font-bold'
                        href="https://github.com/banyancomputer/banyan-cli"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        {messages.useCLI}
                    </a>
                </div>
            }
            <div className="flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={close}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    action={upload}
                    text={`${messages.upload}`}
                    disabled={!isUploadDataFilled}
                />
            </div>
        </div>
    );
};
