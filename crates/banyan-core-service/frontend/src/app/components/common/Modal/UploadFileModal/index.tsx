import React, { useMemo, useState } from 'react';
import { useIntl } from 'react-intl';

import { Select } from '@components/common/Select';
import { AddNewOption } from '@components/common/Select/AddNewOption';
import { CreateBucketModal } from '@components/common/Modal/CreateBucketModal';
import { FolderSelect } from '@components/common/FolderSelect';
import { SubmitButton } from '@components/common/SubmitButton';

import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useFilesUpload } from '@/app/contexts/filesUpload';

import { Upload } from '@static/images/buckets';

export const UploadFileModal: React.FC<{ bucket?: Bucket | null; folder?: BrowserObject; path: string[] }> = ({ bucket, folder, path }) => {
    const { buckets } = useTomb();
    const { openModal, closeModal } = useModal();
    const { setFiles, uploadFiles, files } = useFilesUpload();
    const { messages } = useIntl();
    const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(bucket || null);
    const [selectedFolder, setSelectedFolder] = useState<string[]>(path);
    const isUploadDataFilled = useMemo(() => Boolean(selectedBucket && files.length), [selectedBucket, files]);

    const selectBucket = (bucket: Bucket) => {
        setSelectedBucket(bucket);
    };

    const selectFolder = (option: string[]) => {
        setSelectedFolder(option);
    };

    const handleChange = async(event: React.ChangeEvent<HTMLInputElement>) => {
        if (!event.target.files) { return; }

        setFiles(Array.from(event.target.files).map(file => ({ file, isUploaded: false })));
    };

    const handleDrop = async(event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();

        if (!event.dataTransfer.files) { return; }

        setFiles(Array.from(event.dataTransfer.files).map(file => ({ file, isUploaded: false })));
    };

    const handleDrag = async(event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();
    };

    const upload = async() => {
        if (!files.length) { return; }
        try {
            closeModal();
            ToastNotifications.uploadProgress();
            await uploadFiles(selectedBucket!, selectedFolder.length ? selectedFolder : [], folder);
        } catch (error: any) {
            ToastNotifications.error(`${messages.uploadError}`, `${messages.tryAgain}`, upload);
        };
    };

    const addNewBucket = () => {
        openModal(<CreateBucketModal />, () => openModal(<UploadFileModal bucket={selectedBucket} path={path} />));
    };

    return (
        <div className="w-modal flex flex-col gap-4">
            <div>
                <h4 className="text-m font-semibold ">{`${messages.uploadFiles}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.chooseFilesToUpload}`}
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
                {files.length ?
                    <React.Fragment >
                        {files.map(file =>
                            <span
                                className="w-full overflow-hidden text-ellipsis whitespace-nowrap"
                                key={file.file.name}
                            >
                                {file.file.name}
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
                    multiple={true}
                    className="hidden"
                    onChange={handleChange}
                />
            </label>
            <div className="flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <SubmitButton
                    action={upload}
                    text={`${messages.upload}`}
                    disabled={!isUploadDataFilled}
                />
            </div>
        </div>
    );
};
