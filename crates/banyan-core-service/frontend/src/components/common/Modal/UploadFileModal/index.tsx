import React, { useMemo, useState } from 'react';
import { useIntl } from 'react-intl';

import { useTomb } from '@/contexts/tomb';
import { useModal } from '@/contexts/modals';
import { Select } from '../../Select';
import { AddNewOption } from '../../Select/AddNewOption';
import { CreateBucketModal } from '../CreateBucketModal';

import { Upload } from '@static/images/buckets';
import { Bucket } from '@/lib/interfaces/bucket';
import { FolderSelect } from '../../FolderSelect';

export const UploadFileModal: React.FC<{ bucket?: Bucket }> = ({ bucket }) => {
    const { buckets, uploadFile } = useTomb();
    const { openModal, closeModal } = useModal();
    const { messages } = useIntl();
    const [selectedBucket, setSelectedBucket] = useState(bucket?.id || '');
    const [selectedFolder, setSelectedFolder] = useState<string[]>([]);
    const [file, setFIle] = useState<File | null>(null);
    const isUploadDataFilled = useMemo(() => Boolean(selectedBucket && file), [selectedBucket, file])

    const selectBucket = (option: string) => {
        setSelectedBucket(option);
    };

    const selectFolder = (option: string[]) => {
        setSelectedFolder(option);
    };

    const handleChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
        if (!event.target.files) { return; }

        setFIle(Array.from(event.target.files)[0]);
    };

    const upload = async () => {
        if (!file) return;
        try {
            const arrayBuffer = await file.arrayBuffer();
            await uploadFile(selectedBucket, selectedFolder.length ? selectedFolder : [], file.name, arrayBuffer);
            closeModal();
        } catch (error: any) {
            console.log(error);

        };
    };

    const addNewBucket = () => {
        openModal(<CreateBucketModal />, () => openModal(<UploadFileModal />));
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
                        options={buckets.map(bucket => ({ value: bucket.id, label: bucket.name }))}
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
                        selectedBucket={selectedBucket}
                    />
                </div>
            }
            <label className="mt-10 flex flex-col items-center justify-center gap-4 px-6 py-4 border-2 border-c rounded-xl  text-xs cursor-pointer">
                {file ?
                    <span>{file.name}</span>
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
