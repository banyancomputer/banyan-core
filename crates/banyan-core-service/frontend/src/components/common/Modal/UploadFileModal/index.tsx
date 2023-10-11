import React, { useEffect, useMemo, useState } from 'react';
import { useIntl } from 'react-intl';

import { Select } from '../../Select';
import { AddNewOption } from '../../Select/AddNewOption';
import { CreateBucketModal } from '../CreateBucketModal';
import { FolderSelect } from '../../FolderSelect';

import { Bucket } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';
import { useTomb } from '@/contexts/tomb';
import { ToastNotifications } from '@/utils/toastNotifications';
import { useFolderLocation } from '@/hooks/useFolderLocation';
import { useFilesUpload } from '@/contexts/filesUpload';

import { Upload } from '@static/images/buckets';

export const UploadFileModal: React.FC<{ bucket?: Bucket | null }> = ({ bucket }) => {
    const { buckets } = useTomb();
    const folderLocation = useFolderLocation();
    const { openModal, closeModal } = useModal();
    const { setFiles, uploadFiles, files } = useFilesUpload()
    const { messages } = useIntl();
    const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(bucket || null);
    const [selectedFolder, setSelectedFolder] = useState<string[]>([]);
    const isUploadDataFilled = useMemo(() => Boolean(selectedBucket && files.length), [selectedBucket, files]);

    const selectBucket = (bucket: Bucket) => {
        setSelectedBucket(bucket);
    };

    const selectFolder = (option: string[]) => {
        setSelectedFolder(option);
    };

    const handleChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
        if (!event.target.files) { return; }

        setFiles(Array.from(event.target.files).map(file => ({ file, isUploaded: false })));
    };

    const handleDrop = async (event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();

        if (!event.dataTransfer.files) { return; }

        setFiles(Array.from(event.dataTransfer.files).map(file => ({ file, isUploaded: false })));
    };

    const handleDrag = async (event: React.DragEvent<HTMLInputElement | HTMLLabelElement>) => {
        event.preventDefault();
        event.stopPropagation();
    };

    const upload = async () => {
        if (!files.length) { return; }
        try {
            closeModal();
            ToastNotifications.uploadProgress();
            await uploadFiles(selectedBucket!, selectedFolder.length ? selectedFolder : []);
        } catch (error: any) {
            ToastNotifications.error(`${messages.uploadError}`, `${messages.tryAgain}`, upload);
        };
    };

    const addNewBucket = () => {
        openModal(<CreateBucketModal />, () => openModal(<UploadFileModal bucket={selectedBucket} />));
    };

    return (
        <div className="w-modal flex flex-col gap-4">
            <div>
                <h4 className="text-m font-semibold ">{`${messages.uploadFiles}`}</h4>
                <p className="mt-2 text-gray-600">
                    {`${messages.chooseFilesToUpload}`}
                </p>
            </div>
            {
                !bucket &&
                <div>
                    <span className="inline-block mb-1 text-xs font-normal">{`${messages.selectBucket}`}:</span>
                    <Select
                        selectedOption={selectedBucket}
                        onChange={selectBucket}
                        options={buckets.filter(bucket => bucket.bucketType !== 'backup').map(bucket => ({ value: bucket, label: bucket.name }))}
                        placeholder={`${messages.selectBucket}`}
                        initialOption={<AddNewOption label={`${messages.createNewBucket}`} action={addNewBucket} />}
                    />
                </div>
            }
            {(selectedBucket || bucket) &&
                <div>
                    <span className="inline-block mb-1 text-xs font-normal">{`${messages.selectFolder}`}:</span>
                    <FolderSelect
                        onChange={selectFolder}
                        selectedBucket={selectedBucket!}
                    />
                </div>
            }
            <label
                className="mt-10 flex flex-col items-center justify-center gap-4 px-6 py-4 border-2 border-inputBorder rounded-xl  text-xs cursor-pointer"
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
                        <span className="text-gray-600">
                            <b className="text-gray-900">{`${messages.clickToUpload}`} </b>
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
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary flex-grow py-3 px-4"
                    onClick={upload}
                    disabled={!isUploadDataFilled}
                >
                    {`${messages.upload}`}
                </button>
            </div>
        </div>
    );
};
