import React, { useEffect, useMemo, useState } from 'react';

import { Select } from '@components/common/Select';
import { AddNewOption } from '@components/common/Select/AddNewOption';
import { CreateBucketModal } from '@components/common/Modal/CreateBucketModal';
import { FolderSelect } from '@components/common/FolderSelect';
import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useFilesUpload } from '@/app/contexts/filesUpload';
import { useAppSelector } from '@/app/store';

import { Upload } from '@static/images/buckets';

export const UploadFileModal: React.FC<{ bucket?: Bucket | null; folder?: BrowserObject; path: string[] }> = ({ bucket, folder, path }) => {
    const { buckets } = useTomb();
    const { openModal, closeModal } = useModal();
    const { setFiles, uploadFiles, files } = useFilesUpload();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.uploadFile);
    const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(bucket || null);
    const [selectedFolder, setSelectedFolder] = useState<string[]>(path);
    const isUploadDataFilled = useMemo(() => Boolean(selectedBucket && files.length), [selectedBucket, files]);
    const [previewFiles, setPreviewFiles] = useState<File[]>([]);

    const selectBucket = (bucket: Bucket) => {
        setSelectedBucket(bucket);
    };

    const selectFolder = (option: string[]) => {
        setSelectedFolder(option);
    };

    const handleChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
        if (!event.target.files) { return; }

        setFiles(Array.from(event.target.files).map(file => ({ file, status: 'pending' })));
        setPreviewFiles(Array.from(event.target.files));
    };

    const handleDrop = async (event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();

        if (!event.dataTransfer.files) { return; }

        setFiles(Array.from(event.dataTransfer.files).slice(0, 1).map(file => ({ file, status: 'pending' })));
        setPreviewFiles(Array.from(event.dataTransfer.files).slice(0, 1));
    };

    const handleDrag = async (event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();
    };

    const upload = async () => {
        if (!files.length) { return; }
        try {
            closeModal();
            await uploadFiles(selectedBucket!, selectedFolder.length ? selectedFolder : [], folder);
        } catch (error: any) {
            ToastNotifications.error(`${messages.uploadError}`, `${messages.tryAgain}`, upload);
        };
    };

    const addNewBucket = () => {
        openModal(<CreateBucketModal />, () => openModal(<UploadFileModal bucket={selectedBucket} path={path} />));
    };

    useEffect(() => {
        return () => {
            setPreviewFiles([]);
        };
    }, []);

    return (
        <div className="w-modal flex flex-col gap-4">
            <div>
                <h4 className="text-m font-semibold ">{`${messages.title}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.subtitle}`}
                </p>
            </div>
            {
                !bucket &&
                <div>
                    <span className="inline-block mb-1 text-xs font-normal">{`${messages.selectDrive}`}:</span>
                    <Select
                        selectedOption={selectedBucket}
                        onChange={selectBucket}
                        options={buckets.filter(bucket => bucket.bucketType !== 'backup').map(bucket => ({ value: bucket, label: bucket.name }))}
                        placeholder={`${messages.selectDrive}`}
                        initialOption={<AddNewOption label={`${messages.createNewDrive}`} action={addNewBucket} />}
                    />
                </div>
            }
            {(selectedBucket || bucket) &&
                <div>
                    <span className="inline-block mb-1 text-xs font-normal">{`${messages.selectFolder}`}:</span>
                    <FolderSelect
                        onChange={selectFolder}
                        selectedBucket={selectedBucket!}
                        path={path}
                    />
                </div>
            }
            <label
                className="mt-10 flex flex-col items-center justify-center gap-4 px-6 py-4 border-2 border-border-darken rounded-xl  text-xs cursor-pointer"
                onDrop={handleDrop}
                onDragOver={handleDrag}
            >
                {previewFiles.length ?
                    <React.Fragment >
                        {previewFiles.map(file =>
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
                        <span className="text-text-600">
                            <b className="text-text-900">{`${messages.clickToUpload}`} </b>
                            {`${messages.orDragAndDrop}`}
                        </span>
                    </>
                }
                <input
                    type="file"
                    multiple={false}
                    className="hidden"
                    onChange={handleChange}
                />
            </label>
            <div className="flex items-center gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
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
